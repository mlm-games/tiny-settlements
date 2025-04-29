# Main.gd - Complete rewrite
extends Node

const CardScene = preload("uid://din71ct0bjm3d")
const CardDefs = preload("uid://drx32as7xbxxr")

# --- Game State ---
var game_over: bool = false
var gardener_card: Card = null
var current_gardener_focus: float = 100.0
var max_gardener_focus: float = 100.0
var gardener_action_cost: float = 5.0

var biodiversity_score: int = 0 
var tracked_species: Dictionary = {} 

# --- Nodes ---
@onready var game_board: Node2D = %GameBoard
@onready var focus_recharge_timer: Timer = %FocusRechargeTimer
@onready var nutrient_spawn_timer: Timer = %NutrientSpawnTimer
@onready var passive_process_timer: Timer = %PassiveProcessTimer
@onready var waste_check_timer: Timer = %WasteCheckTimer
@onready var ui: CanvasLayer = %UI
@onready var tutorial_manager: TutorialManager = %TutorialManager

# --- Card Management ---
var active_cards: Array[Card] = []

# --- Tunables ---
@export var gardener_base_action_time: float = 2.0
@export var gardener_harvest_time: float = 3.0
@export var gardener_clean_time: float = 4.0
@export var focus_recharge_rate: float = 3.0
@export var board_size = Rect2(50, 50, 900, 600)
@export var max_slugs_before_waste: int = 5

# --- Signals ---
signal tutorial_card_spawned(card)
signal tutorial_drag_ended(card_a, card_b)
signal tutorial_action_complete(target_card, action_data)

func _ready():
	# Connect signals
	focus_recharge_timer.timeout.connect(_on_focus_recharge_timer_timeout)
	nutrient_spawn_timer.timeout.connect(_on_nutrient_spawn_timer_timeout)
	passive_process_timer.timeout.connect(_on_passive_process_timer_timeout)
	waste_check_timer.timeout.connect(_on_waste_check_timer_timeout)
	
	start_game()

func start_game():
	# Clear existing cards
	for card in get_tree().get_nodes_in_group("cards"):
		card.queue_free()
	active_cards.clear()
	tracked_species.clear()
	biodiversity_score = 0
	game_over = false
	gardener_card = null
	
	# Initial Spawn
	gardener_card = spawn_card(CardDefs.CardType.GARDENER, Vector2(100, 300))
	spawn_card(CardDefs.CardType.BIO_SUBSTRATE, Vector2(250, 200))
	spawn_card(CardDefs.CardType.BIO_SUBSTRATE, Vector2(250, 400))
	spawn_card(CardDefs.CardType.SPORE_POD, Vector2(400, 200))
	spawn_card(CardDefs.CardType.NUTRIENT_SLIME, Vector2(400, 300))
	spawn_card(CardDefs.CardType.NUTRIENT_SLIME, Vector2(400, 400))
	
	# Find initial cards for tutorial
	var initial_spore = null
	var initial_substrate = null
	var initial_nutrient = null
	for card in active_cards:
		if card.card_type == CardDefs.CardType.SPORE_POD: initial_spore = card
		if card.card_type == CardDefs.CardType.BIO_SUBSTRATE: initial_substrate = card
		if card.card_type == CardDefs.CardType.NUTRIENT_SLIME: initial_nutrient = card
		if initial_spore and initial_substrate and initial_nutrient: break

	# Start tutorial if references found
	if tutorial_manager and is_instance_valid(gardener_card) and is_instance_valid(initial_spore) and is_instance_valid(initial_substrate) and is_instance_valid(initial_nutrient):
		tutorial_manager.call_deferred("start_tutorial", self, gardener_card, initial_spore, initial_substrate, initial_nutrient)
	else:
		print("Tutorial prerequisites not met, skipping tutorial.")
		if tutorial_manager: tutorial_manager.end_tutorial()

	# Setup Gardener State
	if gardener_card:
		max_gardener_focus = CardDefs.get_property(CardDefs.CardType.GARDENER, "max_focus", 100.0)
		current_gardener_focus = CardDefs.get_property(CardDefs.CardType.GARDENER, "initial_focus", max_gardener_focus)
		gardener_action_cost = CardDefs.get_property(CardDefs.CardType.GARDENER, "action_cost", 5.0)
		gardener_card.card_properties.focus = current_gardener_focus
		gardener_card.card_properties.max_focus = max_gardener_focus
		gardener_card.update_label()

	# Update UI
	ui.update_biodiversity(biodiversity_score)
	ui.update_gardener_focus(current_gardener_focus, max_gardener_focus)
	ui.clear_status()

	# Start timers
	focus_recharge_timer.start()
	nutrient_spawn_timer.start()
	passive_process_timer.start()
	waste_check_timer.start()

# --- Card Spawning and Management ---

func spawn_card(type: CardDefs.CardType, position: Vector2) -> Card:
	if game_over and type != CardDefs.CardType.GENESIS_BLOOM: return null

	var new_card = CardScene.instantiate() as Card
	new_card.card_type = type

	# Special handling for Gardener
	if type == CardDefs.CardType.GARDENER:
		if gardener_card != null: new_card.queue_free(); return null
		gardener_card = new_card

	# Keep cards on board
	new_card.global_position = position.clamp(
		board_size.position, 
		board_size.end - new_card.get_node("CollisionShape2D").shape.size
	)
	
	game_board.add_child(new_card)
	active_cards.append(new_card)
	new_card.add_to_group("cards")
	
	# Emit tutorial signal
	if is_instance_valid(new_card) and tutorial_manager and tutorial_manager.is_active:
		emit_signal("tutorial_card_spawned", new_card)

	# Connect signals
	new_card.drag_started.connect(_on_card_drag_started)
	new_card.drag_ended.connect(_on_card_drag_ended)
	new_card.action_complete.connect(_on_gardener_action_complete)
	new_card.passive_action_ready.connect(_on_passive_action_ready)
	new_card.self_destruct.connect(_on_card_destroyed.bind(new_card))
	new_card.tree_exiting.connect(_on_card_destroyed.bind(new_card))

	# Update Biodiversity Score
	check_and_update_biodiversity(new_card)

	return new_card

func check_and_update_biodiversity(card: Card):
	# Check if it's a "mature" stage we want to count
	var is_mature_species = card.card_type in [
		CardDefs.CardType.BASIC_FUNGI, CardDefs.CardType.MATURE_VINE,
		CardDefs.CardType.MATURE_FLUTTERWING, CardDefs.CardType.SYMBIOTIC_ALGAE,
		CardDefs.CardType.GRAZING_SLUG, CardDefs.CardType.GENESIS_BLOOM
	]
	
	if is_mature_species:
		if not tracked_species.has(card.card_type):
			tracked_species[card.card_type] = 0
		tracked_species[card.card_type] += 1

		if tracked_species[card.card_type] == 1: # First time this species appeared
			biodiversity_score += 1
			ui.update_biodiversity(biodiversity_score)

func remove_from_biodiversity(card: Card):
	if tracked_species.has(card.card_type):
		tracked_species[card.card_type] -= 1
		if tracked_species[card.card_type] <= 0:
			tracked_species.erase(card.card_type)
			biodiversity_score = tracked_species.size()
			ui.update_biodiversity(biodiversity_score)

# --- Core Interaction Logic ---

func _on_card_drag_started(card: Card):
	# Nothing special needed here
	pass

func _on_card_drag_ended(card: Card, dropped_on_cards: Array[Card]):
	if game_over or card.is_working: return
	
	# Discard outside board
	if not board_size.has_point(card.global_position):
		return
		
	if dropped_on_cards.is_empty():
		return
		
	var target_card = dropped_on_cards[0]
	
	# Emit tutorial signal
	if tutorial_manager and tutorial_manager.is_active:
		emit_signal("tutorial_drag_ended", card, target_card)
	
	# Process the interaction
	handle_card_interaction(card, target_card)

func handle_card_interaction(card_a: Card, card_b: Card):
	if card_a.is_working or card_b.is_working:
		print("One of the cards is busy")
		return
		
	var type_a = card_a.card_type
	var type_b = card_b.card_type
	
	print("Processing interaction: ", CardDefs.get_label(type_a), " + ", CardDefs.get_label(type_b))
	
	# --- GARDENER INTERACTIONS ---
	if type_a == CardDefs.CardType.GARDENER:
		handle_gardener_interaction(card_a, card_b)
		return
		
	# --- SEED + SUBSTRATE ---
	if CardDefs.is_seed_or_spore(type_a) and CardDefs.is_substrate(type_b):
		print(CardDefs.get_label(type_a), " placed on ", CardDefs.get_label(type_b), ". Needs Gardener action.")
		card_a.global_position = card_b.global_position + Vector2(0, -10)
		return
		
	# --- NUTRIENT + PLANT ---
	if CardDefs.is_nutrient(type_a) and (CardDefs.is_plant(type_b) or CardDefs.is_seed_or_spore(type_b) and card_b.is_planted):
		var needed_nutrient = CardDefs.get_property(type_b, "needs_nutrient", CardDefs.CardType.NONE)
		if type_a == needed_nutrient:
			print(CardDefs.get_label(type_a), " placed on ", CardDefs.get_label(type_b), ". Needs Gardener action.")
			card_a.global_position = card_b.global_position + Vector2(randf_range(-10,10), -10)
		else:
			print(CardDefs.get_label(type_b), " doesn't need this nutrient type now.")
		return
		
	# --- MULCH + SUBSTRATE ---
	if type_a == CardDefs.CardType.RICH_MULCH and type_b == CardDefs.CardType.BIO_SUBSTRATE:
		print("Rich Mulch placed on Bio-Substrate. Needs Gardener action.")
		card_a.global_position = card_b.global_position + Vector2(0, -10)
		return
		
	# --- RECIPES ---
	handle_recipe_combination(card_a, card_b)

func handle_gardener_interaction(gardener: Card, target: Card):
	if gardener.is_working:
		print("Gardener is busy!")
		return
		
	var cost = gardener_action_cost
	var target_type = target.card_type
	
	# --- PLANTING SEEDS ---
	if CardDefs.is_seed_or_spore(target_type) and not target.is_planted:
		var substrate = find_nearby_card_type(target, [CardDefs.is_substrate])
		if substrate:
			if can_afford_action(cost):
				print("Gardener plants ", CardDefs.get_label(target_type))
				spend_gardener_focus(cost)
				gardener.is_working = true
				gardener.sprite.modulate = Color(0.8,0.8,0.8)
				gardener.play_feedback_animation("working")
				target.start_gardener_action(gardener_base_action_time)
				target.card_properties.gardener_action_data = {"action": "plant"}
				gardener.global_position = target.global_position + Vector2(0, -40)
			else:
				print("Not enough focus to plant!")
		else:
			print("No substrate found nearby for planting")
			
	# --- APPLYING NUTRIENTS ---
	elif CardDefs.is_nutrient(target_type):
		var plant = find_nearby_plant_needing_nutrient(target)
		if plant:
			if can_afford_action(cost):
				print("Gardener applies ", CardDefs.get_label(target_type), " to ", CardDefs.get_label(plant.card_type))
				spend_gardener_focus(cost)
				gardener.is_working = true
				gardener.sprite.modulate = Color(0.8,0.8,0.8)
				gardener.play_feedback_animation("working")
				plant.start_gardener_action(gardener_base_action_time)
				plant.card_properties.gardener_action_data = {"action": "apply_nutrient", "source": target}
				gardener.global_position = plant.global_position + Vector2(0, -40)
			else:
				print("Not enough focus to apply nutrient!")
		else:
			print("No plant found that needs this nutrient nearby")
			
	# --- HARVESTING ---
	elif CardDefs.is_plant(target_type):
		if CardDefs.get_property(target_type, "yields_on_harvest", CardDefs.CardType.NONE) != CardDefs.CardType.NONE:
			if can_afford_action(cost):
				print("Gardener harvests ", CardDefs.get_label(target_type))
				spend_gardener_focus(cost)
				gardener.is_working = true
				gardener.sprite.modulate = Color(0.8,0.8,0.8)
				gardener.play_feedback_animation("working")
				target.start_gardener_action(gardener_harvest_time)
				target.card_properties.gardener_action_data = {"action": "harvest"}
				gardener.global_position = target.global_position + Vector2(0, -40)
			else:
				print("Not enough focus to harvest!")
				
	# --- CLEANING WASTE ---
	elif target_type == CardDefs.CardType.WASTE_TOXIN:
		if can_afford_action(cost):
			print("Gardener cleans Waste Toxin")
			spend_gardener_focus(cost)
			gardener.is_working = true
			gardener.sprite.modulate = Color(0.8,0.8,0.8)
			gardener.play_feedback_animation("working")
			target.start_gardener_action(gardener_clean_time)
			target.card_properties.gardener_action_data = {"action": "clean"}
			gardener.global_position = target.global_position + Vector2(0, -40)
		
# --- Continue from handle_gardener_interaction function ---

	# --- APPLYING MULCH ---
	elif target_type == CardDefs.CardType.RICH_MULCH:
		var substrate = find_nearby_card_type(target, [CardDefs.CardType.BIO_SUBSTRATE])
		if substrate:
			if can_afford_action(cost):
				print("Gardener applies Mulch to Substrate")
				spend_gardener_focus(cost)
				gardener.is_working = true
				gardener.sprite.modulate = Color(0.8,0.8,0.8)
				gardener.play_feedback_animation("working")
				substrate.start_gardener_action(gardener_base_action_time)
				substrate.card_properties.gardener_action_data = {"action": "upgrade_substrate", "source": target}
				gardener.global_position = substrate.global_position + Vector2(0, -40)
			else:
				print("Not enough focus to apply mulch!")
		else:
			print("No substrate found nearby for mulch")

func handle_recipe_combination(card_a: Card, card_b: Card):
	var type_a = card_a.card_type
	var type_b = card_b.card_type
	var spawn_pos = (card_a.global_position + card_b.global_position) / 2
	
	# --- VINE SEED RECIPE ---
	if (type_a == CardDefs.CardType.NUTRIENT_SLIME and type_b == CardDefs.CardType.PROCESSED_NUTRIENTS) or \
	   (type_b == CardDefs.CardType.NUTRIENT_SLIME and type_a == CardDefs.CardType.PROCESSED_NUTRIENTS):
		spawn_card(CardDefs.CardType.VINE_SEED, spawn_pos)
		card_a.consume(false)
		card_b.consume(false)
		return true
	
	# --- FLUTTERWING SPORE RECIPE ---
	if (type_a == CardDefs.CardType.PROCESSED_NUTRIENTS and type_b == CardDefs.CardType.BASIC_FUNGI) or \
	   (type_b == CardDefs.CardType.PROCESSED_NUTRIENTS and type_a == CardDefs.CardType.BASIC_FUNGI):
		spawn_card(CardDefs.CardType.FLUTTERWING_SPORE, spawn_pos)
		card_a.consume(false)
		card_b.consume(false)
		return true
	
	# --- GRAZING SLUG EGG RECIPE ---
	if (type_a == CardDefs.CardType.RICH_MULCH and type_b == CardDefs.CardType.BASIC_FUNGI) or\
	   (type_b == CardDefs.CardType.RICH_MULCH and type_a == CardDefs.CardType.BASIC_FUNGI):
		spawn_card(CardDefs.CardType.GRAZING_SLUG_EGG, spawn_pos)
		card_a.consume(false)
		card_b.consume(false)
		return true
	
	# --- APEX SPORE RECIPE ---
	if (type_a == CardDefs.CardType.LUMINA_CRYSTAL and type_b == CardDefs.CardType.SYMBIOTIC_ALGAE) or \
	   (type_b == CardDefs.CardType.LUMINA_CRYSTAL and type_a == CardDefs.CardType.SYMBIOTIC_ALGAE):
		spawn_card(CardDefs.CardType.APEX_SPORE, spawn_pos)
		card_a.consume(false)
		card_b.consume(false)
		return true
		
	return false

# --- Gardener Action Handlers ---

func _on_gardener_action_complete(target_card: Card):
	# This is called when the action timer on the TARGET card finishes
	if game_over or not gardener_card: return

	print("Gardener finished action on: ", CardDefs.get_label(target_card.card_type))
	gardener_card.is_working = false
	gardener_card.sprite.modulate = Color.WHITE
	gardener_card.play_feedback_animation("complete")

	# Get the action data
	var action_data = target_card.card_properties.get("gardener_action_data", {})
	
	# Emit tutorial signal
	if tutorial_manager and tutorial_manager.is_active and not action_data.is_empty():
		emit_signal("tutorial_action_complete", target_card, action_data)

	if action_data.is_empty(): return

	var action = action_data.get("action", "")
	var source_card = action_data.get("source", null)

	match action:
		"plant":
			target_card.is_planted = true
			print(CardDefs.get_label(target_card.card_type), " planted.")
			trigger_growth_check(target_card)
			
		"apply_nutrient":
			if is_instance_valid(source_card): 
				source_card.consume(false)
			trigger_growth_check(target_card)
			
		"harvest":
			var yield_type = CardDefs.get_property(target_card.card_type, "yields_on_harvest", CardDefs.CardType.NONE)
			if yield_type != CardDefs.CardType.NONE:
				spawn_card(yield_type, target_card.global_position + Vector2(randf_range(40, 60), randf_range(-10, 10)))
			target_card.consume(true)
			
		"clean":
			target_card.consume(true)
			
		"upgrade_substrate":
			if is_instance_valid(source_card): 
				source_card.consume(false)
			var pos = target_card.global_position
			target_card.consume(false)
			spawn_card(CardDefs.CardType.FERTILE_SUBSTRATE, pos)

	# Clean up action data
	target_card.card_properties.erase("gardener_action_data")

# --- Passive Action Handlers ---

func _on_passive_action_ready(card: Card, action_type: String):
	if game_over or not is_instance_valid(card): return

	print("Passive action ready on ", CardDefs.get_label(card.card_type), ": ", action_type)
	card.play_feedback_animation("passive_pulse")

	match action_type:
		"produce":
			handle_passive_production(card)
			
		"pollinate":
			handle_pollination(card)
			
		"hatch":
			handle_egg_hatching(card)
			
		"eat":
			handle_slug_eating(card)
			
		"grow":
			handle_growth(card)

	# Update card label
	card.update_label()

func handle_passive_production(card: Card):
	var produce_type = CardDefs.get_property(card.card_type, "produces_passively", CardDefs.CardType.NONE)
	if produce_type == CardDefs.CardType.NONE:
		return
		
	# Check if conditions still met
	var can_produce = true
	var needed_substrate = CardDefs.get_property(card.card_type, "needs_substrate", CardDefs.CardType.NONE)
	
	if needed_substrate != CardDefs.CardType.NONE:
		if not find_nearby_card_type(card, [needed_substrate]):
			can_produce = false
			print(CardDefs.get_label(card.card_type), " cannot produce, missing ", CardDefs.get_label(needed_substrate))

	if can_produce:
		spawn_card(produce_type, card.global_position + Vector2(randf_range(40, 60), randf_range(-10, 10)))
		# Restart the timer
		var interval = CardDefs.get_property(card.card_type, "passive_interval", 10.0)
		card.start_passive_timer("produce", interval)
	else:
		card.stop_passive_timer()

func handle_pollination(card: Card):
	card.is_pollinated = true
	card.needs_pollination = false
	card.update_label()
	
	# Find and free the pollinator
	var pollinator = find_nearby_card_type(card, [CardDefs.CardType.MATURE_FLUTTERWING])
	if pollinator:
		pollinator.is_working = false
		
	# Spawn a fertilized pod
	spawn_card(CardDefs.CardType.FERTILIZED_VINE_POD, card.global_position + Vector2(0, 20))

func handle_egg_hatching(card: Card):
	var pos = card.global_position
	card.consume(false)
	spawn_card(CardDefs.CardType.GRAZING_SLUG, pos)

func handle_slug_eating(card: Card):
	var target_food = find_nearby_card_type(card, [CardDefs.get_property(card.card_type, "eats", CardDefs.CardType.NONE)])
	if target_food:
		target_food.consume(true)
		# Slug can now produce mulch
		card.start_passive_timer("produce")
		trigger_passive_eating_check(card)
	else:
		card.stop_passive_timer()

func handle_growth(card: Card):
	var next_stage = CardDefs.CardType.NONE
	
	# Define all growth transitions
	if card.card_type == CardDefs.CardType.SPORE_POD: 
		next_stage = CardDefs.CardType.BASIC_FUNGI
	elif card.card_type == CardDefs.CardType.VINE_SEED: 
		next_stage = CardDefs.CardType.YOUNG_VINE
	elif card.card_type == CardDefs.CardType.YOUNG_VINE: 
		next_stage = CardDefs.CardType.MATURE_VINE
	elif card.card_type == CardDefs.CardType.FLUTTERWING_SPORE: 
		next_stage = CardDefs.CardType.FLUTTERWING_LARVA
	elif card.card_type == CardDefs.CardType.FLUTTERWING_LARVA: 
		next_stage = CardDefs.CardType.MATURE_FLUTTERWING
	elif card.card_type == CardDefs.CardType.FERTILIZED_VINE_POD: 
		next_stage = CardDefs.CardType.SYMBIOTIC_ALGAE
	elif card.card_type == CardDefs.CardType.APEX_SPORE: 
		next_stage = CardDefs.CardType.GROWING_APEX
	elif card.card_type == CardDefs.CardType.GROWING_APEX: 
		next_stage = CardDefs.CardType.GENESIS_BLOOM

	if next_stage != CardDefs.CardType.NONE:
		var pos = card.global_position
		var was_planted = card.is_planted
		card.consume(false)
		var new_card = spawn_card(next_stage, pos)
		
		# Transfer planted state
		if new_card and was_planted:
			new_card.is_planted = true
			
		if card.card_type == CardDefs.CardType.FLUTTERWING_SPORE:
			await get_tree().create_timer(20).timeout
			for c in active_cards:
				if c.card_type == CardDefs.CardType.FLUTTERWING_LARVA:
					trigger_growth_check(c)
			
			
		if next_stage == CardDefs.CardType.GENESIS_BLOOM:
			win_game()
	else:
		print("Growth finished but no next stage defined for ", CardDefs.get_label(card.card_type))

# --- Growth Logic ---

func trigger_growth_check(plant_card: Card):
	if not is_instance_valid(plant_card): return

	var current_type = plant_card.card_type
	
	# Special case for Spore Pod - grows automatically after planting
	if current_type == CardDefs.CardType.SPORE_POD and plant_card.is_planted:
		print("Basic Fungi starts growing.")
		plant_card.start_passive_timer("grow", 5.0)
		return
		
	# Special case for Flutterwing Larva - grows automatically
	if current_type == CardDefs.CardType.FLUTTERWING_LARVA:
		print("Flutterwing Larva starts maturing.")
		plant_card.start_passive_timer("grow", 10.0)
		return
		
	# For all other plants, check if nutrient was applied and conditions are met
	var needs_nutrient_type = CardDefs.get_property(current_type, "needs_nutrient", CardDefs.CardType.NONE)
	var needs_substrate_type = CardDefs.get_property(current_type, "needs_substrate", CardDefs.CardType.NONE)
	
	# Check if nutrient was applied
	var nutrient_applied = plant_card.card_properties.get("gardener_action_data", {}).get("action", "") == "apply_nutrient"
	var source_card = plant_card.card_properties.get("gardener_action_data", {}).get("source", null)
	
	var correct_nutrient = true
	if needs_nutrient_type != CardDefs.CardType.NONE and is_instance_valid(source_card):
		if source_card.card_type != needs_nutrient_type:
			correct_nutrient = false
			
	var substrate_ok = true
	if needs_substrate_type != CardDefs.CardType.NONE:
		if not find_nearby_card_type(plant_card, [needs_substrate_type]):
			substrate_ok = false
			
	if nutrient_applied and correct_nutrient and substrate_ok:
		print(CardDefs.get_label(current_type), " starts growing.")
		plant_card.start_passive_timer("grow", 8.0)
	else:
		print("Growth conditions not met for ", CardDefs.get_label(current_type))

# --- Timer Handlers ---

func _on_focus_recharge_timer_timeout():
	if game_over or not gardener_card: return
	
	if not gardener_card.is_working and not gardener_card.is_dragging:
		current_gardener_focus = min(max_gardener_focus, current_gardener_focus + focus_recharge_rate)
		gardener_card.card_properties.focus = current_gardener_focus
		gardener_card.update_label()
		ui.update_gardener_focus(current_gardener_focus, max_gardener_focus)

func _on_nutrient_spawn_timer_timeout():
	if game_over: return
	
	var spawn_pos = Vector2(
		randf_range(board_size.position.x, board_size.end.x),
		randf_range(board_size.position.y, board_size.end.y)
	)
	spawn_card(CardDefs.CardType.NUTRIENT_SLIME, spawn_pos)

func _on_passive_process_timer_timeout():
	if game_over: return
	
	for card in active_cards:
		if not is_instance_valid(card) or not card.passive_action_timer.is_stopped():
			continue
			
		check_passive_production(card)
		
		check_passive_pollination(card)
		
		check_passive_hatching(card)
		
		check_passive_eating(card)

func check_passive_production(card: Card):
	var produce_type = CardDefs.get_property(card.card_type, "produces_passively", CardDefs.CardType.NONE)
	if produce_type == CardDefs.CardType.NONE:
		return
		
	var can_produce = true
	var needed_substrate = CardDefs.get_property(card.card_type, "needs_substrate", CardDefs.CardType.NONE)
	
	if needed_substrate != CardDefs.CardType.NONE and not find_nearby_card_type(card, [needed_substrate]):
		can_produce = false
		
	if can_produce:
		var interval = CardDefs.get_property(card.card_type, "passive_interval", 10.0)
		card.start_passive_timer("produce", interval)

func check_passive_pollination(card: Card):
	if card.card_type == CardDefs.CardType.MATURE_VINE and card.needs_pollination and not card.is_pollinated:
		var pollinator = find_nearby_card_type(card, [CardDefs.CardType.MATURE_FLUTTERWING])
		if pollinator and not pollinator.is_working:
			print("Pollinator found for Vine!")
			card.start_passive_timer("pollinate", 5.0)
			pollinator.is_working = true
			pollinator.global_position = card.global_position + Vector2(0, -30)

func check_passive_hatching(card: Card):
	if card.card_type == CardDefs.CardType.GRAZING_SLUG_EGG:
		var needed_nearby = CardDefs.get_property(card.card_type, "needs_nearby", CardDefs.CardType.NONE)
		if needed_nearby != CardDefs.CardType.NONE and find_nearby_card_type(card, [needed_nearby]):
			card.start_passive_timer("hatch", 8.0)

func check_passive_eating(card: Card):
	trigger_passive_eating_check(card)

func trigger_passive_eating_check(card: Card):
	var eats_type = CardDefs.get_property(card.card_type, "eats", CardDefs.CardType.NONE)
	if eats_type != CardDefs.CardType.NONE and card.passive_action_timer.is_stopped():
		var food = find_nearby_card_type(card, [eats_type])
		if food:
			print(CardDefs.get_label(card.card_type), " starts eating ", CardDefs.get_label(food.card_type))
			card.start_passive_timer("eat", 6.0)

func _on_waste_check_timer_timeout():
	if game_over: return
	
	var slug_count = 0
	for card in active_cards:
		if is_instance_valid(card) and card.card_type == CardDefs.CardType.GRAZING_SLUG:
			slug_count += 1
			
	if slug_count >= max_slugs_before_waste:
		print("Imbalance detected! Too many slugs.")
		var spawn_pos = Vector2(
			randf_range(board_size.position.x, board_size.end.x),
			randf_range(board_size.position.y, board_size.end.y)
		)
		spawn_card(CardDefs.CardType.WASTE_TOXIN, spawn_pos)

# --- Helper Functions ---

func can_afford_action(cost: float) -> bool:
	if gardener_card and current_gardener_focus >= cost:
		return true
	print("Not enough Gardener Focus!")
	if gardener_card: 
		gardener_card.play_feedback_animation("error_pulse")
	return false

func spend_gardener_focus(cost: float):
	if gardener_card:
		current_gardener_focus -= cost
		gardener_card.card_properties.focus = current_gardener_focus
		gardener_card.update_label()
		ui.update_gardener_focus(current_gardener_focus, max_gardener_focus)

func find_nearby_card_type(source_card: Card, types_to_find: Array, max_distance: float = 80.0) -> Card:
	if not is_instance_valid(source_card):
		return null
		
	# First try direct overlaps
	var overlaps = source_card.get_overlapping_cards()
	
	for card in overlaps:
		if not is_instance_valid(card):
			continue
			
		# Check for function reference
		if types_to_find.size() == 1 and typeof(types_to_find[0]) == TYPE_CALLABLE:
			if types_to_find[0].call(card.card_type):
				return card
		elif card.card_type in types_to_find:
			return card
	
	# If no direct overlap, try distance-based approach
	var closest_card = null
	var closest_distance = max_distance
	
	for other_card in active_cards:
		if not is_instance_valid(other_card) or other_card == source_card:
			continue
			
		# Check if card is of the right type
		var type_matches = false
		if types_to_find.size() == 1 and typeof(types_to_find[0]) == TYPE_CALLABLE:
			type_matches = types_to_find[0].call(other_card.card_type)
		else:
			type_matches = other_card.card_type in types_to_find
			
		if type_matches:
			# Check distance
			var distance = source_card.global_position.distance_to(other_card.global_position)
			if distance < closest_distance:
				closest_card = other_card
				closest_distance = distance
	
	return closest_card

func find_nearby_plant_needing_nutrient(nutrient_card: Card) -> Card:
	var nutrient_type = nutrient_card.card_type
	var closest_plant = null
	var closest_distance = 80.0
	
	for card in active_cards:
		if not is_instance_valid(card):
			continue
			
		# Check if it's a plant or planted seed that needs this nutrient
		var is_valid_target = (CardDefs.is_plant(card.card_type) || 
							  (CardDefs.is_seed_or_spore(card.card_type) && card.is_planted))
		
		if is_valid_target:
			# Check if this nutrient is what the plant needs
			var needed_nutrient = CardDefs.get_property(card.card_type, "needs_nutrient", CardDefs.CardType.NONE)
			if nutrient_type == needed_nutrient:
				# Check distance
				var distance = card.global_position.distance_to(nutrient_card.global_position)
				if distance < closest_distance:
					closest_plant = card
					closest_distance = distance
	
	return closest_plant

func _on_card_destroyed(card: Card):
	if card in active_cards:
		active_cards.erase(card)
	remove_from_biodiversity(card)
	
	if card == gardener_card:
		if not game_over: 
			lose_game("The Gardener vanished!")
		gardener_card = null

# --- Game Flow ---

func win_game():
	if game_over: return
	
	print("Game Won!")
	game_over = true
	ui.show_win_message()
	
	# Stop timers
	focus_recharge_timer.stop()
	nutrient_spawn_timer.stop()
	passive_process_timer.stop()
	waste_check_timer.stop()

func lose_game(reason: String):
	if game_over: return
	
	print("Game Lost! Reason: ", reason)
	game_over = true
	ui.show_lose_message(reason)
	
	# Stop timers
	focus_recharge_timer.stop()
	nutrient_spawn_timer.stop()
	passive_process_timer.stop()
	waste_check_timer.stop()
	
	if gardener_card:
		gardener_card.is_working = true
		gardener_card.sprite.modulate = Color(0.5, 0.5, 0.5)

func _process(delta):
	if Input.is_action_just_pressed("ui_accept") and game_over:
		print("Restarting game...")
		start_game()
