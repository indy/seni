/*
 *  Copyright (C) 2020 Inderjit Gill <email@indy.io>
 *
 *  This file is part of Seni
 *
 *  Seni is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Affero General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  Seni is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU Affero General Public License for more details.
 *
 *  You should have received a copy of the GNU Affero General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

///<reference path='SeniMode.ts'/>


interface IHashGalleryItems {
    [key: number] : GalleryItem;
}

interface GalleryItem {
    id: number;
    name: string;
    script: string;
};

class State {
    public highResolution: [number, number];
    public populationSize: number;
    public mutationRate: number;

    public currentMode: SeniMode;

    public galleryLoaded: boolean;
    public galleryOldestToDisplay: number;
    public galleryItems: IHashGalleryItems;
    public galleryDisplaySize: number; // the number of gallery sketches to display everytime 'load more' is clicked

    public previouslySelectedGenotypes: string[];
    public selectedIndices: Array<number>;
    public scriptId: number | undefined;
    public script: string | undefined;
    public genotypes: Array<string>;
    public traits: Array<string>;

    public genotype: string;

    static createInitialState(): State {
        let s = new this();

        s.highResolution = [2048, 2048];
        s.populationSize = 24;
        s.mutationRate = 0.1;
        s.currentMode = SeniMode.gallery;
        s.galleryLoaded = false;
        s.galleryOldestToDisplay = 9999;
        s.galleryItems = {};
        s.galleryDisplaySize = 20; // was 20
        s.previouslySelectedGenotypes = [];
        s.selectedIndices = [];
        s.scriptId = undefined;
        s.script = undefined;
        s.genotypes = [];
        s.traits = [];
        s.genotype = "";

        return s;
    }

    // note: the State parameter is going to be a plain JS object
    // so a 'real' State object needs to be constructed
    static createStateFromObject(obj: State): State {
        let s = new this();

        s.highResolution = obj.highResolution;
        s.populationSize = obj.populationSize;
        s.mutationRate = obj.mutationRate;
        s.currentMode = obj.currentMode;
        s.galleryLoaded = obj.galleryLoaded;
        s.galleryOldestToDisplay = obj.galleryOldestToDisplay;
        s.galleryItems = obj.galleryItems;
        s.galleryDisplaySize = obj.galleryDisplaySize;
        s.previouslySelectedGenotypes = obj.previouslySelectedGenotypes;
        s.selectedIndices = obj.selectedIndices;
        s.scriptId = obj.scriptId;
        s.script = obj.script;
        s.genotypes = obj.genotypes;
        s.traits = obj.traits;
        s.genotype = obj.genotype;

        return s;
    }

    clone(): State {
        let s = new State();

        s.highResolution = this.highResolution;
        s.populationSize = this.populationSize;
        s.mutationRate = this.mutationRate;
        s.currentMode = this.currentMode;
        s.galleryLoaded = this.galleryLoaded;
        s.galleryOldestToDisplay = this.galleryOldestToDisplay;
        s.galleryItems = this.galleryItems;
        s.galleryDisplaySize = this.galleryDisplaySize;
        s.previouslySelectedGenotypes = this.previouslySelectedGenotypes;
        s.selectedIndices = this.selectedIndices;
        s.scriptId = this.scriptId;
        s.script = this.script;
        s.genotypes = this.genotypes;
        s.traits = this.traits;
        s.genotype = this.genotype;

        return s;
    }

    constructor() {
    }
}
