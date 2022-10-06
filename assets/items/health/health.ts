type Health = {
  0: i32;
};
const Health: BevyType<Health> = { typeName: "punchy::damage::Health" };
type Stats = {
  max_health: i32;
  movement_speed: f32;
};
const Stats: BevyType<Stats> = { typeName: "punchy::fighter::Stats" };

export default {
  postUpdate() {
    // The `getItemGrabEvents()` method will return all events where a fighter grabbed the item that
    // this script is associated with.
    const grabEvents = punchy.getItemGrabEvents();

    const fighterQuery = world.query(Health, Stats);
    for (const event of grabEvents) {
      const fighter = event.fighter;

      const [health, stats] = fighterQuery.get(fighter);

      health[0] = stats.max_health;
    }
  },
}
