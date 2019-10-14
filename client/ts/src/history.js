// --------------------------------------------------------------------------------
// history

function senModeAsString(state) {
  const mode = state.currentMode;

  switch (mode) {
  case SeniMode.gallery:
    return 'gallery';
  case SeniMode.edit:
    return state.scriptId;
  case SeniMode.evolve:
    return 'evolve';
  default:
    return 'error unknown SeniMode value';
  }
}

function buildState(appState) {
  const state = appState;
  const uri = `#${senModeAsString(state)}`;

  return [state, uri];
}

const SeniHistory = {
  pushState: function(appState) {
    const [state, uri] = buildState(appState);
    log('historyPushState', state);
    history.pushState(state, null, uri);
  },
  replaceState: function(appState) {
    const [state, uri] = buildState(appState);
    log('historyReplace', state);
    history.replaceState(state, null, uri);
  },
  restoreState: function(state) {
    log('historyRestore', state);
    return state;
  }
};
