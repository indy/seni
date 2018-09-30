package main

// based on https://golang.org/doc/articles/wiki/

import (
	"encoding/json"
	"errors"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"strconv"
	"strings"
)

type GalleryItem struct {
	Id    int
	Name  string
	Image string
}

var galleryItems []GalleryItem

func galleryListHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	body, err := ioutil.ReadFile("server/gallery.json")
	if err != nil {
		return
	}
	fmt.Fprintf(w, string(body))
}

func galleryHandler(w http.ResponseWriter, r *http.Request) {
	parts := strings.Split(r.URL.Path, "/")
	last := parts[len(parts)-1]
	id, err := strconv.Atoi(last)
	if err != nil {
		http.Redirect(w, r, "/gallery", http.StatusFound)
		return
	}

	galleryItem, err := getGalleryItem(id)
	if err != nil {
		http.Redirect(w, r, "/gallery", http.StatusFound)
		return
	}

	filename := "server/seni/" + galleryItem.Name + ".seni"
	body, err := ioutil.ReadFile(filename)
	if err != nil {
		http.Redirect(w, r, "/gallery", http.StatusFound)
		return
	}

	fmt.Fprintf(w, string(body))
}

func getGalleryItem(id int) (*GalleryItem, error) {
	for i := 0; i < len(galleryItems); i++ {
		if galleryItems[i].Id == id {
			return &galleryItems[i], nil
		}
	}
	return nil, errors.New("unable to find id")
}

func parseGallery() ([]GalleryItem, error) {
	body, err := ioutil.ReadFile("server/gallery.json")
	if err != nil {
		return nil, err
	}
	var m []GalleryItem
	dec := json.NewDecoder(strings.NewReader(string(body)))
	if err := dec.Decode(&m); err != nil {
		return nil, err
	}
	return m, nil
}

func maxAgeHandler(seconds int, h http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Add("Cache-Control", fmt.Sprintf("max-age=%d, public, must-revalidate, proxy-revalidate", seconds))
		h.ServeHTTP(w, r)
	})
}

func main() {

	const BaseName = "/"

	galleryItems2, err := parseGallery()
	if err != nil {
		fmt.Println("fooked")
		log.Fatal(err)
	}
	galleryItems = galleryItems2

	http.HandleFunc(BaseName+"gallery", galleryListHandler)
	http.HandleFunc(BaseName+"gallery/", galleryHandler)

	fs := http.FileServer(http.Dir("assets"))
	http.Handle(BaseName, maxAgeHandler(0, http.StripPrefix(BaseName, fs)))

	fs = http.FileServer(http.Dir("dist"))
	http.Handle(BaseName+"dist/", http.StripPrefix(BaseName+"dist/", fs))

	fmt.Printf("Serving localhost:3210\n")
	http.ListenAndServe(":3210", nil)
}
