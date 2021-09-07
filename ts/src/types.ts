export type Mayor = {
	name: string;
	role: string;
	perks: Perk[];
	special: boolean;
};

export type Perk = {
	name: string;
	description: string;
};

export interface HypixelResponse {
	success: boolean;
}

export interface BazaarData extends HypixelResponse {
	lastUpdated: number;
	products: {
		[itemID: string]: BazaarProduct;
	};
}

export interface BazaarProduct {
	product_id: string;
	sell_summary: {
		amount: number;
		pricePerUnit: number;
		orders: number;
	}[];
	buy_summary: {
		amount: number;
		pricePerUnit: number;
		orders: number;
	}[];
	quick_status: {
		productId: string;
		sellPrice: number;
		sellVolume: number;
		sellMovingWeek: number;
		sellOrders: number;
		buyPrice: number;
		buyVolume: number;
		buyMovingWeek: number;
		buyOrders: number;
	};
}

export interface TotalAuctions extends HypixelResponse {
	page: number;
	totalPages: number;
	totalAuctions: number;
	lastUpdated: number;
	auctions: Auction[];
}

export interface Auction {
	_id: string;
	uuid: string;
	auctioneer: string;
	profile_id: string;
	coop: string[];
	start: number;
	end: number;
	item_name: string;
	item_lore: string;
	extra: string;
	category: 'armor' | 'blocks' | 'consumables' | 'misc' | 'weapon';
	tier:
		| 'COMMON'
		| 'UNCOMMON'
		| 'RARE'
		| 'EPIC'
		| 'LEGENDARY'
		| 'MYTHIC'
		| 'SUPREME'
		| 'SPECIAL'
		| 'VERY SPECIAL';
	starting_bid: number;
	item_bytes: string;
	claimed: boolean;
	claimed_bidders: [];
	highest_bid_amount: number;
	bin?: boolean;
	bids: Bid[];
}

export interface Bid {
	auction_id: string;
	bidder: string;
	profile_id: string;
	amount: number;
	timestamp: number;
}

export type Jerry = {
	nextSwitch: number;
	mayor: Mayor;
	perks: Perk[];
};
