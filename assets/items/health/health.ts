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
    const grabEvents = punchy.getItemGrabEvents();

    const fighterQuery = world.query(Health, Stats);
    for (const event of grabEvents) {
      const fighter = event.fighter;

      const [health, stats] = fighterQuery.get(fighter);

      health[0] = stats.max_health;
    }
  },
}
