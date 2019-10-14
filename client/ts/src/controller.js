// --------------------------------------------------------------------------------
// controller

const actionSetMode = 'SET_MODE';
const actionSetGenotype = 'SET_GENOTYPE';
const actionSetScript = 'SET_SCRIPT';
const actionSetScriptId = 'SET_SCRIPT_ID';
const actionSetSelectedIndices = 'SET_SELECTED_INDICES';
const actionInitialGeneration = 'INITIAL_GENERATION';
const actionNextGeneration = 'NEXT_GENERATION';
const actionShuffleGeneration = 'SHUFFLE_GENERATION';
const actionSetState = 'SET_STATE';
const actionSetGalleryItems = "SET_GALLERY_ITEMS";
const actionGalleryOldestToDisplay = 'GALLERY_OLDEST_TO_DISPLAY';

function createInitialState() {
  return {
    // the resolution of the high res image
    highResolution: [2048, 2048], // [4096, 4096],
    populationSize: 24,
    mutationRate: 0.1,

    currentMode: SeniMode.gallery,

    galleryLoaded: false,
    galleryOldestToDisplay: 9999,
    galleryItems: {},
    galleryDisplaySize: 20,     // the number of gallery sketchs to display everytime 'load more' is clicked

    previouslySelectedGenotypes: [],
    selectedIndices: [],
    scriptId: undefined,
    script: undefined,
    genotypes: [],
    traits: [],

    genotype: undefined,
  };
}

class Controller {
  constructor(initialState) {
    this.currentState = initialState;
  }

  cloneState(state) {
    const clone = {};

    clone.highResolution = state.highResolution;
    clone.populationSize = state.populationSize;
    clone.mutationRate = state.mutationRate;

    clone.currentMode = state.currentMode;

    clone.galleryLoaded = state.galleryLoaded;
    clone.galleryOldestToDisplay = state.galleryOldestToDisplay;
    clone.galleryItems = state.galleryItems;
    clone.galleryDisplaySize = state.galleryDisplaySize;

    clone.previouslySelectedGenotypes = state.previouslySelectedGenotypes;
    clone.selectedIndices = state.selectedIndices;
    clone.scriptId = state.scriptId;
    clone.script = state.script;
    clone.genotypes = state.genotypes;
    clone.traits = state.traits;

    return clone;
  }

  async applySetMode(state, { mode }) { // note: this doesn't need to be async?
    const newState = this.cloneState(state);
    newState.currentMode = mode;

    this.currentState = newState;
    return this.currentState;
  }

  async applySetGenotype(state, { genotype }) {
    const newState = this.cloneState(state);
    newState.genotype = genotype;

    this.currentState = newState;
    return this.currentState;
  }

  async applySetScript(state, { script }) {

    const newState = this.cloneState(state);
    newState.script = script;

    const { validTraits, traits } = await Job.request(jobBuildTraits, {
      script: newState.script
    });

    if (validTraits) {
      newState.traits = traits;
    } else {
      newState.traits = [];
    }

    this.currentState = newState;
    return this.currentState;
  }

  async applySetScriptId(state, { id }) {
    const newState = this.cloneState(state);
    newState.scriptId = id;

    this.currentState = newState;
    return this.currentState;
  }

  async applySetSelectedIndices(state, { selectedIndices }) {
    const newState = this.cloneState(state);
    newState.selectedIndices = selectedIndices || [];

    this.currentState = newState;
    return this.currentState;
  }

  // todo: should populationSize be passed in the action?
  async applyInitialGeneration(state) {
    const newState = this.cloneState(state);
    let { genotypes } = await Job.request(jobInitialGeneration, {
      traits: newState.traits,
      populationSize: newState.populationSize
    });

    newState.genotypes = genotypes;
    newState.previouslySelectedGenotypes = [];
    newState.selectedIndices = [];

    this.currentState = newState;
    return this.currentState;
  }

  async applyGalleryOldestToDisplay(state, { oldestId }) {
    const newState = this.cloneState(state);
    newState.galleryOldestToDisplay = oldestId;

    this.currentState = newState;
    return this.currentState;
  }

  async applySetGalleryItems(state, { galleryItems }) {
    const newState = this.cloneState(state);

    newState.galleryItems = {};
    galleryItems.forEach(item => {
      newState.galleryItems[item.id] = item;
    });
    if (galleryItems.length === 0)  {
      console.error("galleryItems is empty?");
    }

    newState.galleryLoaded = true;
    newState.galleryOldestToDisplay = galleryItems[0].id;

    this.currentState = newState;
    return this.currentState;
  }

  async applyShuffleGeneration(state, { rng }) {
    const newState = this.cloneState(state);
    const prev = newState.previouslySelectedGenotypes;

    if (prev.length === 0) {
      const s = await this.applyInitialGeneration(newState);

      this.currentState = s;
      return this.currentState;
    } else {
      const { genotypes } = await Job.request(jobNewGeneration, {
        genotypes: prev,
        populationSize: newState.populationSize,
        traits: newState.traits,
        mutationRate: newState.mutationRate,
        rng
      });

      newState.genotypes = genotypes;
      newState.selectedIndices = [];

      this.currentState = newState;
      return this.currentState;
    }
  }

  async applyNextGeneration(state, { rng }) {
    const newState = this.cloneState(state);
    const pg = newState.genotypes;
    const selectedIndices = newState.selectedIndices;
    const selectedGenos = [];

    for (let i = 0; i < selectedIndices.length; i++) {
      selectedGenos.push(pg[selectedIndices[i]]);
    }

    const { genotypes } = await Job.request(jobNewGeneration, {
      genotypes: selectedGenos,
      populationSize: newState.populationSize,
      traits: newState.traits,
      mutationRate: newState.mutationRate,
      rng
    });

    const previouslySelectedGenotypes = genotypes.slice(0, selectedIndices.length);

    newState.genotypes = genotypes;
    newState.previouslySelectedGenotypes = previouslySelectedGenotypes;
    newState.selectedIndices = [];

    this.currentState = newState;
    return this.currentState;
  }

  async applySetState(newState) {
    this.currentState = newState;
    return this.currentState;
  }

  logMode(mode) {
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
    log(`${actionSetMode}: ${name}`);
  }

  reducer(state, action) {
    switch (action.__type) {
    case actionSetMode:
      if (logToConsole) {
        this.logMode(action.mode);
      }
      return this.applySetMode(state, action);
    case actionSetGenotype:
      // SET_GENOTYPE is only used during the download dialog rendering
      // when the render button is clicked on an image in the evolve gallery
      //
      return this.applySetGenotype(state, action);
    case actionSetScript:
      return this.applySetScript(state, action);
    case actionSetScriptId:
      return this.applySetScriptId(state, action);
    case actionSetSelectedIndices:
      return this.applySetSelectedIndices(state, action);
    case actionInitialGeneration:
      return this.applyInitialGeneration(state);
    case actionNextGeneration:
      return this.applyNextGeneration(state, action);
    case actionShuffleGeneration:
      return this.applyShuffleGeneration(state, action);
    case actionSetState:
      log(`${actionSetState}: ${action.state}`);
      return this.applySetState(action.state);
    case actionGalleryOldestToDisplay:
      return this.applyGalleryOldestToDisplay(state, action);
    case actionSetGalleryItems:
      return this.applySetGalleryItems(state, action);
    default:
      return this.applySetState(state);
    }
  }

  getState() {
    return this.currentState;
  }

  dispatch(action, data) {
    if (data === undefined) {
      data = {};
    }
    data.__type = action;

    log(`dispatch: action = ${data.__type}`);
    return this.reducer(this.currentState, data);
  }
}
