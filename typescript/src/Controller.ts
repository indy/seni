/*
 *  Seni
 *  Copyright (C) 2020 Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

///<reference path='Log.ts'/>
///<reference path='Job.ts'/>
///<reference path='SeniMode.ts'/>
///<reference path='State.ts'/>

enum Action {
    SetMode,
    SetGenotype,
    SetScript,
    SetScriptId,
    SetSelectedIndices,
    InitialGeneration,
    NextGeneration,
    ShuffleGeneration,
    SetState,
    SetGalleryItems,
    GalleryOldestToDisplay,
}

class Controller {
    currentState: State;

    constructor(initialState: State) {
        this.currentState = initialState;
    }

    async applySetMode(state: State, { mode }: { mode: SeniMode }) { // note: this doesn't need to be async?
        const newState = state.clone();
        newState.currentMode = mode;

        this.currentState = newState;
        return this.currentState;
    }

    async applySetGenotype(state: State, { genotype }: { genotype: Array<any> }) {
        const newState = state.clone();
        newState.genotype = genotype;

        this.currentState = newState;
        return this.currentState;
    }

    async applySetScript(state: State, { script }: { script: string | undefined }) {
        const newState = state.clone();
        newState.script = script;

        const { validTraits, traits } = await Job.request(JobType.jobBuildTraits, {
            script: newState.script
        }, undefined);

        if (validTraits) {
            newState.traits = traits;
        } else {
            newState.traits = [];
        }

        this.currentState = newState;
        return this.currentState;
    }

    async applySetScriptId(state: State, { id }: { id: number | undefined }) { // todo: is this undefined required?
        const newState = state.clone();
        newState.scriptId = id;

        this.currentState = newState;
        return this.currentState;
    }

    async applySetSelectedIndices(state: State, { selectedIndices }: { selectedIndices: Array<number> }) {
        const newState = state.clone();
        newState.selectedIndices = selectedIndices || [];

        this.currentState = newState;
        return this.currentState;
    }

    // todo: should populationSize be passed in the action?
    async applyInitialGeneration(state: State) {
        const newState = state.clone();
        let { genotypes } = await Job.request(JobType.jobInitialGeneration, {
            traits: newState.traits,
            populationSize: newState.populationSize
        }, undefined);

        newState.genotypes = genotypes;
        newState.previouslySelectedGenotypes = [];
        newState.selectedIndices = [];

        this.currentState = newState;
        return this.currentState;
    }

    async applyGalleryOldestToDisplay(state: State, { oldestId }: { oldestId: number }) {
        const newState = state.clone();
        newState.galleryOldestToDisplay = oldestId;

        this.currentState = newState;
        return this.currentState;
    }

    async applySetGalleryItems(state: State, { galleryItems }: { galleryItems: any }) {
        const newState = state.clone();

        newState.galleryItems = {};
        galleryItems.forEach((item: any) => {
            // Log.log(`setGalleryItems: ${item.id}`);
            newState.galleryItems[item.id] = item;
        });
        if (galleryItems.length === 0)  {
            Log.error("galleryItems is empty?");
        }

        newState.galleryLoaded = true;
        newState.galleryOldestToDisplay = galleryItems[0].id;

        this.currentState = newState;
        return this.currentState;
    }

    async applyShuffleGeneration(state: State, { rng }: { rng: any }) {
        const newState = state.clone();
        const prev = newState.previouslySelectedGenotypes;

        if (prev.length === 0) {
            const s = await this.applyInitialGeneration(newState);

            this.currentState = s;
            return this.currentState;
        } else {
            const { genotypes } = await Job.request(JobType.jobNewGeneration, {
                genotypes: prev,
                populationSize: newState.populationSize,
                traits: newState.traits,
                mutationRate: newState.mutationRate,
                rng
            }, undefined);

            newState.genotypes = genotypes;
            newState.selectedIndices = [];

            this.currentState = newState;
            return this.currentState;
        }
    }

    async applyNextGeneration(state: State, { rng }: { rng: any }) {
        const newState = state.clone();
        const pg = newState.genotypes;
        const selectedIndices = newState.selectedIndices;
        const selectedGenos = [];

        for (let i = 0; i < selectedIndices.length; i++) {
            selectedGenos.push(pg[selectedIndices[i]]);
        }

        const { genotypes } = await Job.request(JobType.jobNewGeneration, {
            genotypes: selectedGenos,
            populationSize: newState.populationSize,
            traits: newState.traits,
            mutationRate: newState.mutationRate,
            rng
        }, undefined);

        const previouslySelectedGenotypes = genotypes.slice(0, selectedIndices.length);

        newState.genotypes = genotypes;
        newState.previouslySelectedGenotypes = previouslySelectedGenotypes;
        newState.selectedIndices = [];

        this.currentState = newState;
        return this.currentState;
    }

    async applySetState(newState: State) {
        this.currentState = newState;
        return this.currentState;
    }

    logMode(mode: SeniMode) {
        let name = '';
        switch (mode) {
            case SeniMode.gallery:
                name = 'gallery';
                break;
            case SeniMode.edit:
                name = 'edit';
                break;
            case SeniMode.evolve:
                name = 'evolve';
                break;
            default:
                name = 'unknown';
                break;
        }
        Log.log(`${Action.SetMode}: ${name}`);
    }

    reducer(state: State, action: any) {
        switch (action.__type) {
            case Action.SetMode:
                if (Log.logToConsole) {
                    this.logMode(action.mode);
                }
                return this.applySetMode(state, action);
            case Action.SetGenotype:
                // SET_GENOTYPE is only used during the download dialog rendering
                // when the render button is clicked on an image in the evolve gallery
                //
                return this.applySetGenotype(state, action);
            case Action.SetScript:
                return this.applySetScript(state, action);
            case Action.SetScriptId:
                return this.applySetScriptId(state, action);
            case Action.SetSelectedIndices:
                return this.applySetSelectedIndices(state, action);
            case Action.InitialGeneration:
                return this.applyInitialGeneration(state);
            case Action.NextGeneration:
                return this.applyNextGeneration(state, action);
            case Action.ShuffleGeneration:
                return this.applyShuffleGeneration(state, action);
            case Action.SetState:
                Log.log(`${Action.SetState}: ${action.state}`);
                return this.applySetState(action.state);
            case Action.GalleryOldestToDisplay:
                return this.applyGalleryOldestToDisplay(state, action);
            case Action.SetGalleryItems:
                return this.applySetGalleryItems(state, action);
            default:
                return this.applySetState(state);
        }
    }

    getState() {
        return this.currentState;
    }

    dispatch(action: Action, data: any) {
        if (data === undefined) {
            data = {};
        }
        data.__type = action;

        Log.log(`dispatch: action = ${data.__type}`);
        return this.reducer(this.currentState, data);
    }
}
