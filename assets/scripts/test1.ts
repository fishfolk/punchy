class Test1 implements ScriptSystems {
  counter: number = 0;

  update() {
    if (this.counter == 0) {
      Punchy.log("First update for my script!", "info");
    }

    Punchy.log(`Script update #${this.counter}`, "trace");
    this.counter++;
  }
}

/**
 * This is the only required function in scripts. It must return a type implementing
 * `ScriptSystems`.
 */
function init(): ScriptSystems {
  return new Test1();
}
