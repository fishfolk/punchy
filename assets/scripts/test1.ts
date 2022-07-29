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

export default new Test1();
