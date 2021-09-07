import ax from 'axios';
import axRetry from 'axios-retry';
import { transformItemData } from '@zikeji/hypixel';
import _ from 'lodash';
import { Auction, BazaarData, TotalAuctions } from './types';
import fs from 'fs-extra';
import fastJson from 'fast-json-stringify';
import path from 'path';

const stringify = fastJson({});

const axios = ax.create({
	headers: {
		'User-Agent': 'SkytilsBackend/1.0.0',
	},
});

axRetry(axios);

const getAuctionURL = (page = 0) =>
	`https://api.hypixel.net/skyblock/auctions?page=${page}&apikey=535df159-07da-4365-b362-3e8b5c6ab221`;

// const data = new Map();
const bins: any = {};
fetchAuctions().then(() => {
	fs.writeFileSync(
		path.join(process.cwd(), 'lowestbins.json'),
		JSON.stringify(bins)
	);
});
async function fetchAuctions() {
	let d: TotalAuctions = (await axios.get(getAuctionURL(1))).data;
	let auc = d.auctions;

	for (let i = 2; i <= d.totalPages; i++) {
		let j: TotalAuctions = await (
			await axios.get(getAuctionURL(i), {
				validateStatus: (s) => (s >= 200 && s < 300) || s == 404,
			})
		).data;
		if (!j.success) continue;
		auc.push(...j.auctions);
	}
	await filterLowestBIN(auc);
	await addBazaarData();
}

async function addBazaarData() {
	let bazaarData: BazaarData = await (
		await axios.get('https://api.hypixel.net/skyblock/bazaar')
	).data;
	if (!bazaarData.success) return;
	let whitelisted = [
		'HOT_POTATO_BOOK',
		'FUMING_POTATO_BOOK',
		'RECOMBOBULATOR_3000',
	];
	for (let item of whitelisted) {
		let product = bazaarData.products[item];
		if (product) {
			bins[item] = Math.round(
				product.sell_summary[0]?.pricePerUnit || product.quick_status.buyPrice
			);
		}
	}
}

async function filterLowestBIN(auctions: Auction[]) {
	let lowest = _(
		await Promise.all(
			auctions.map(async (auction) => {
				if (!auction.bin) return;
				let itemData = (await transformItemData(auction.item_bytes))[0];
				let nbt = itemData?.tag;
				let count = itemData?.Count || 1;
				let extraAttr = nbt?.ExtraAttributes;
				let skyblockId = extraAttr?.id;
				if (!extraAttr || !skyblockId) return;
				switch (skyblockId) {
					case 'PET':
						if (!extraAttr.petInfo) break;
						//@ts-expect-error
						let petInfo = JSON.parse(extraAttr['petInfo']);
						skyblockId = `PET-${petInfo['type']}-${petInfo['tier']}`;
						break;
					case 'ENCHANTED_BOOK':
						let enchantments = extraAttr.enchantments || {};
						let enchantmentNames = Object.keys(enchantments);
						if (enchantmentNames.length === 1)
							skyblockId = `ENCHANTED_BOOK-${enchantmentNames[0].toUpperCase()}-${
								enchantments[enchantmentNames[0]]
							}`;
						break;
					case 'POTION':
						let potionName = extraAttr.potion;
						let potionLevel = extraAttr.potion_level;
						skyblockId = `POTION${
							potionName ? `-${potionName.toUpperCase()}` : ''
						}${potionLevel ? `-${potionLevel}` : ''}${
							extraAttr.enhanced ? '-ENHANCED' : ''
						}${extraAttr.extended ? '-EXTENDED' : ''}${
							extraAttr.splash ? '-SPLASH' : ''
						}`;
						break;
					case 'RUNE':
						let runes = extraAttr.runes || {};
						let runeNames = Object.keys(runes);
						if (runeNames.length === 1)
							skyblockId = `RUNE-${runeNames[0].toUpperCase()}-${
								runes[runeNames[0]]
							}`;
						break;
				}
				return {
					id: skyblockId,
					price: auction.starting_bid / count,
				};
			})
		)
	)
		.omitBy(_.isNil)
		.groupBy((x) => x?.id)
		.map((group) => _.minBy(group, 'price'))
		.value();
	for (let item of lowest) {
		if (!item) continue;
		bins[item.id] = item.price;
	}
}
