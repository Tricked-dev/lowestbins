package main

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"math/rand"
	"net/http"
	"os"
	"strconv"
	"time"

	"github.com/gorilla/mux"
)

// Book struct (Model)
type Book struct {
	ID     string  `json:"id"`
	Isbn   string  `json:"isbn"`
	Title  string  `json:"title"`
	Author *Author `json:"author"`
}

// Author struct
type Author struct {
	Firstname string `json:"firstname"`
	Lastname  string `json:"lastname"`
}
type Item struct {
	UUID             string        `json:"uuid"`
	Auctioneer       string        `json:"auctioneer"`
	ProfileID        string        `json:"profile_id"`
	Coop             []string      `json:"coop"`
	Start            int64         `json:"start"`
	End              int64         `json:"end"`
	ItemName         string        `json:"item_name"`
	ItemLore         string        `json:"item_lore"`
	Extra            string        `json:"extra"`
	Category         string        `json:"category"`
	Tier             string        `json:"tier"`
	StartingBid      int           `json:"starting_bid"`
	ItemBytes        string        `json:"item_bytes"`
	Claimed          bool          `json:"claimed"`
	ClaimedBidders   []interface{} `json:"claimed_bidders"`
	HighestBidAmount int           `json:"highest_bid_amount"`
	Bin              bool          `json:"bin,omitempty"`
	Bids             []interface{} `json:"bids"`
}
type Auction struct {
	Success       bool   `json:"success"`
	Page          int    `json:"page"`
	TotalPages    int    `json:"totalPages"`
	TotalAuctions int    `json:"totalAuctions"`
	LastUpdated   int64  `json:"lastUpdated"`
	Auctions      []Item `json:"auctions"`
}

type AuctionItem struct {
	id    string
	price int
}

var data map[string]int

// Init books var as a slice Book struct
var books []Book

const (
	url = "https://api.hypixel.net/skyblock/auctions?page=1"
)

// Get all books
func getBooks(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	response, err := http.Get(url)
	if err != nil {
		fmt.Print(err.Error())
		os.Exit(1)
	}

	responseData, err := ioutil.ReadAll(response.Body)
	if err != nil {
		log.Fatal(err)
	}
	println(string(responseData))
	var responseObject AuctionResponse
	json.Unmarshal(responseData, &responseObject)

	// var actions AuctionResponse
	// json.Unmarshal(resultBodyStr, &actions)

	json.NewEncoder(w).Encode(responseObject.Auctions)
}

// Get single book
func getBook(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	params := mux.Vars(r) // Gets params
	// Loop through books and find one with the id from the params
	for _, item := range books {
		if item.ID == params["id"] {
			json.NewEncoder(w).Encode(item)
			return
		}
	}
	json.NewEncoder(w).Encode(&Book{})
}

// Add new book
func createBook(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	var book Book
	_ = json.NewDecoder(r.Body).Decode(&book)
	book.ID = strconv.Itoa(rand.Intn(100000000)) // Mock ID - not safe
	books = append(books, book)
	json.NewEncoder(w).Encode(book)
}

// Update book
func updateBook(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	params := mux.Vars(r)
	for index, item := range books {
		if item.ID == params["id"] {
			books = append(books[:index], books[index+1:]...)
			var book Book
			_ = json.NewDecoder(r.Body).Decode(&book)
			book.ID = params["id"]
			books = append(books, book)
			json.NewEncoder(w).Encode(book)
			return
		}
	}
}

// Delete book
func deleteBook(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	params := mux.Vars(r)
	for index, item := range books {
		if item.ID == params["id"] {
			books = append(books[:index], books[index+1:]...)
			break
		}
	}
	json.NewEncoder(w).Encode(books)
}

func getPage(index int) []Item {
	url := "https://api.hypixel.net/skyblock/auctions?page="
	concatenated := fmt.Sprintf("%d%s", url, index)
	response, err := http.Get(concatenated)
	if err != nil {
		fmt.Print(err.Error())
		os.Exit(1)
	}

	responseData, err := ioutil.ReadAll(response.Body)
	if err != nil {
		log.Fatal(err)
	}
	var responseObject Auction
	json.Unmarshal(responseData, &responseObject)
	return responseObject.Auctions
}

func convert(data []Auction) []AuctionItem {
	var returnData []AuctionItem
	for i := 0; i < len(data); i++ {

	}
	return returnData
}

func updatePrices() {

	response, err := http.Get(url)
	if err != nil {
		fmt.Print(err.Error())
		os.Exit(1)
	}

	responseData, err := ioutil.ReadAll(response.Body)
	if err != nil {
		log.Fatal(err)
	}
	var responseObject Auction
	json.Unmarshal(responseData, &responseObject)

}

// Main function
func main() {

	// Init router
	r := mux.NewRouter()

	// Hardcoded data - @todo: add database
	books = append(books, Book{ID: "1", Isbn: "438227", Title: "Book One", Author: &Author{Firstname: "John", Lastname: "Doe"}})
	books = append(books, Book{ID: "2", Isbn: "454555", Title: "Book Two", Author: &Author{Firstname: "Steve", Lastname: "Smith"}})

	// Route handles & endpoints
	r.HandleFunc("/books", getBooks).Methods("GET")
	r.HandleFunc("/books/{id}", getBook).Methods("GET")
	r.HandleFunc("/books", createBook).Methods("POST")
	r.HandleFunc("/books/{id}", updateBook).Methods("PUT")
	r.HandleFunc("/books/{id}", deleteBook).Methods("DELETE")

	// Start server
	log.Fatal(http.ListenAndServe(":8000", r))
	for {
		time.Sleep(5 * time.Minute)
		updatePrices()
	}
}

// Request sample
// {
// 	"isbn":"4545454",
// 	"title":"Book Three",
// 	"author":{"firstname":"Harry","lastname":"White"}
// }
