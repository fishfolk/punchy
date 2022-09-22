type Health = {
  0: number;
};
const Health: BevyType<Health> = { typeName: "punchy::damage::Health" };
type Stats = {
  max_health: i32,
  movement_speed: f32,
};
const Stats: BevyType<Stats> = { typeName: "punchy::fighter::Stats" };

export default {
  post_update() {
    const grabEvents = punchy.getItemGrabEvents();

    for (const event of grabEvents) {
      const fighter = event.fighter;

      const [health, stats] = world.get(fighter, Health, Stats);

      health[0] = stats.max_health;
    }
  },
};
