// --------------------------------------------------------------------------------
// timer

const db = {};
const printPrecision = 2;

function getStats(entry) {
  if (entry.num === 0) {
    return null;
  }
  return {
    current: entry.last,
    average: (entry.sum / entry.num),
    min: entry.min,
    max: entry.max,
    num: entry.num
  };
}


function addTiming(entry, duration) {
  entry.num += 1;
  entry.sum += duration;
  entry.last = duration;
  if (duration < entry.min) {
    entry.min = duration;
  }
  if (duration > entry.max) {
    entry.max = duration;
  }
  return entry;
}

function useDBEntry(id) {
  if (!db[id]) {
    db[id] = {
      id,
      num: 0,
      sum: 0,
      last: 0,
      min: 100000,
      max: 0
    };
  }

  return db[id];
}

function startTiming() {
  if (logToConsole) {
    const before = performance.now();
    // return the 'stop' function
    return (id) => {
      const entry = useDBEntry(id);

      const after = performance.now();
      const duration = after - before;

      addTiming(entry, duration);

      const stats = getStats(entry);

      if (stats) {
        const eid = entry.id;
        const cur = stats.current.toFixed(printPrecision);
        const avg = stats.average.toFixed(printPrecision);
        const min = stats.min.toFixed(printPrecision);
        const max = stats.max.toFixed(printPrecision);
        const num = stats.num;

        const msg1 = `${eid}: ${cur}ms `;
        const msg2 = `(Mean: ${avg}, Min: ${min}, Max: ${max} N:${num})`;

        log(msg1 + msg2);
      }
    };
  } else {
    // do nothing
    return (id) => {};
  }
}

function getTimingEntry(id) {
  return db[id];
}
