/* 
  This is a Punchy script!

  This one is added to the game globally in the default.game.yaml's scripts section. This doesn't do
  anything useful but is used to show how to get started.

  First off, if you open this script in the Punchy repo, you will get typescript auto-completion
  because of the types files and the `tsconfig.json` file in the repo. If you have your own game
  folder, you can copy the `tsconfig.json` and related files to get that working in your project.
*/

/* 
  Until this is automated, in order to access Bevy components in our script, we create these type
  definitions in order to inform TypeScript how to load our component.

  The bevy core types come out-of-the-box, so you only need to add your game's components like this.
*/
type Health = {
  0: i32; // Health is a tuple struct with the number as it's first field.
};
const Health: BevyType<Health> = { typeName: "punchy::damage::Health" };
type Stats = {
  max_health: i32; // Note that these number types are just aliases to JavaScripts `number` type.
  movement_speed: f32;
};
const Stats: BevyType<Stats> = { typeName: "punchy::fighter::Stats" };

// This is a variable local to the script. Each script have it's contents evaluated once so
// variables like this one can be initialized.
let frameIdx = 0;

// The default export is required for every script
export default {
  /**
   * Here we add a function that will be run after the bevy core stage of the same name.
   *
   * The stage names are the same as the Bevy CoreStage names except in camelCase.
   */
  update() {
    // Only log this message on the first frame
    if (frameIdx == 0) {
      // There are global logging functions for trace(), debug(), info(), warn(), and error(). These
      // will be visible in the bevy log with the `js_runtime` target.
      info("Hello from the demo script!");
    }

    // Do this every so many frames
    if (frameIdx % 60 == 0) {
      // There is also a `world` global we can use to get access to the Bevy world. Here we print
      // out the transform of all of our fighters.
      const query = world.query(Health, Stats, Transform);

      if (query.length > 0) {
        info("=== Fighters ===");

        // We loop through all the entities matching the query
        for (const fighter of query) {
          // And extract their components
          const [health, stats, transform] = fighter.components;
          let { x, y, z } = transform.translation;
          // Round values for nicer display
          x = Math.round(x);
          y = Math.round(y);
          z = Math.round(z);

          // Finally, we print the fighter info
          info(
            `Fighter has ${health[0]} health with a speed of ${stats.movement_speed} and is at position (${x}, ${y}, ${z}).`
          );

          // We can also modify the components! For instance, uncomment the following line to increase
          // the fighters walk speed by 10% every 60 frames! 🏃‍♀️

          // stats.movement_speed *= 1.1;
        }
        info("======");
      }
    }

    frameIdx++;
  },
};
