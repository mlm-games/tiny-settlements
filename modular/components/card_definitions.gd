class_name CardDefinitions extends RefCounted

enum CardType {
	NONE,
	GARDENER,
	SEEDLING_PATCH,
	CHRONO_CRYSTAL,

	SUNPETAL_SEED,

	SUNPETAL_SPROUT,
	SUNPETAL_MATURE,

	SUNPETAL_POLLEN,

	PAST_DEW,
	FLEETING_MOMENT,

	CONCENTRATED_DEW_RECIPE,
	CONCENTRATED_DEW,

	TEMPORAL_WEED,

	AEVUM_BLOOM_RECIPE,
	AEVUM_BLOOM
}

const CARD_PROPERTIES = {
	CardType.GARDENER: {"label": "Gardener", "initial_energy": 100.0, "max_energy": 100.0, "action_cost": 5.0},
	CardType.SEEDLING_PATCH: {"label": "Seedling Patch"},
	CardType.CHRONO_CRYSTAL: {"label": "Chrono Crystal", "initial_energy": 300.0},

	CardType.SUNPETAL_SEED: {"label": "Sunpetal Seed"},
	CardType.SUNPETAL_SPROUT: {"label": "Sunpetal Sprout", "growth_stage": 1},
	CardType.SUNPETAL_MATURE: {"label": "Mature Sunpetal", "growth_stage": 2, "yields": CardType.SUNPETAL_POLLEN},

	CardType.SUNPETAL_POLLEN: {"label": "Sunpetal Pollen"},

	CardType.PAST_DEW: {"label": "Past Dew", "growth_power": 1.0},
	CardType.FLEETING_MOMENT: {"label": "Fleeting Moment", "growth_power": 3.0, "lifespan": 10.0},

	CardType.CONCENTRATED_DEW_RECIPE: {"label": "Concentrate Dew\n(Need Gardener)"},
	CardType.CONCENTRATED_DEW: {"label": "Concentrated Dew"},

	CardType.TEMPORAL_WEED: {"label": "Temporal Weed"},

	CardType.AEVUM_BLOOM_RECIPE: {"label": "Cultivate Bloom\n(Need Gardener)"},
	CardType.AEVUM_BLOOM: {"label": "Aevum Bloom\n(Garden Saved!)"}
}

static func get_label(type: CardType) -> String:
	if CARD_PROPERTIES.has(type) and CARD_PROPERTIES[type].has("label"):
		return CARD_PROPERTIES[type].label
	return "Unknown"

static func get_property(type: CardType, prop_name: String, default_value = null):
	if CARD_PROPERTIES.has(type) and CARD_PROPERTIES[type].has(prop_name):
		return CARD_PROPERTIES[type][prop_name]
	return default_value
