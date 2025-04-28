class_name Main extends Node

const CardScene = preload("uid://din71ct0bjm3d")
const CardDefs = preload("uid://drx32as7xbxxr")

var game_over: bool = false
var gardener_card: Card = null
var chrono_crystal_card: Card = null

var current_gardener_energy: float = 0.0
var max_gardener_energy: float = 100.0
var gardener_action_cost: float = 5.0 # Default cost

var current_crystal_energy: float = 300.0
var crystal_drain_rate: float = 1.0 # Energy lost per tick

@onready var game_board: Node2D = $GameBoard
@onready var crystal_drain_timer: Timer = $CrystalDrainTimer
@onready var gardener_recharge_timer: Timer = $GardenerRechargeTimer
@onready var energy_spawn_timer: Timer = $EnergySpawnTimer
#@onready var anomaly_check_timer: Timer = $AnomalyCheckTimer # Add when using 
@onready var ui: CanvasLayer = $UI

var active_cards: Array[Card] = []

@export var plant_growth_time: float = 5.0
@export var harvest_time: float = 3.0
@export var craft_time: float = 4.0
@export var gardener_recharge_rate: float = 4.0 # Energy gained per tick when idle
@export var board_size = Rect2(50, 100, 800, 500)

func _ready():
	crystal_drain_timer.timeout.connect(_on_crystal_drain_timer_timeout)
	gardener_recharge_timer.timeout.connect(_on_gardener_recharge_timer_timeout)
	energy_spawn_timer.timeout.connect(_on_energy_spawn_timer_timeout)
	#anomaly_check_timer.timeout.connect(_on_anomaly_check_timer_timeout) # Connect if using

	start_game()

func start_game():
	# Clear existing cards
	for card in get_tree().get_nodes_in_group("cards"):
		card.queue_free()
	active_cards.clear()
	game_over = false
	gardener_card = null
	chrono_crystal_card = null

	# Initial card spawn
	gardener_card = spawn_card(CardDefs.CardType.GARDENER, Vector2(150, 300))
	chrono_crystal_card = spawn_card(CardDefs.CardType.CHRONO_CRYSTAL, Vector2(150, 150))
	spawn_card(CardDefs.CardType.SEEDLING_PATCH, Vector2(300, 300))
	spawn_card(CardDefs.CardType.PAST_DEW, Vector2(450, 250))
	spawn_card(CardDefs.CardType.PAST_DEW, Vector2(450, 350))
	spawn_card(CardDefs.CardType.SUNPETAL_SEED, Vector2(600, 300))

	if gardener_card:
		max_gardener_energy = CardDefs.get_property(CardDefs.CardType.GARDENER, "max_energy", 100.0)
		current_gardener_energy = CardDefs.get_property(CardDefs.CardType.GARDENER, "initial_energy", max_gardener_energy)
		gardener_action_cost = CardDefs.get_property(CardDefs.CardType.GARDENER, "action_cost", 5.0)
		gardener_card.card_properties.energy = current_gardener_energy # Sync card property
		gardener_card.card_properties.max_energy = max_gardener_energy
		gardener_card.update_label()

	if chrono_crystal_card:
		current_crystal_energy = CardDefs.get_property(CardDefs.CardType.CHRONO_CRYSTAL, "initial_energy", 300.0)
		chrono_crystal_card.card_properties.energy = current_crystal_energy # Sync property
		chrono_crystal_card.update_label()

	ui.update_crystal_energy(current_crystal_energy)
	ui.update_gardener_energy(current_gardener_energy, max_gardener_energy)
	ui.clear_status()

	crystal_drain_timer.start()
	gardener_recharge_timer.start()
	energy_spawn_timer.start()
	#anomaly_check_timer.start()

func spawn_card(type: CardDefs.CardType, position: Vector2) -> Card:
	if game_over and type != CardDefs.CardType.AEVUM_BLOOM: return null # Allow final bloom spawn

	var new_card = CardScene.instantiate() as Card
	# Set type first, which copies base properties
	new_card.card_type = type

	# Special handling for singleton cards
	if type == CardDefs.CardType.GARDENER:
		if gardener_card != null: new_card.queue_free(); return null
		gardener_card = new_card
	elif type == CardDefs.CardType.CHRONO_CRYSTAL:
		if chrono_crystal_card != null: new_card.queue_free(); return null
		chrono_crystal_card = new_card

	new_card.global_position = position
	game_board.add_child(new_card)
	active_cards.append(new_card)
	new_card.add_to_group("cards")

	new_card.drag_started.connect(_on_card_drag_started)
	new_card.drag_ended.connect(_on_card_drag_ended)
	new_card.action_complete.connect(_on_card_action_complete)
	new_card.self_destruct.connect(_on_card_destroyed.bind(new_card)) # Connect self destruct
	new_card.tree_exiting.connect(_on_card_destroyed.bind(new_card))

	return new_card


func _on_card_drag_started(card: Card):
	pass # later add some juice

func _on_card_drag_ended(card: Card, dropped_on_cards: Array[Card]):
	if game_over: return

	# Simple discard outside board
	if not board_size.has_point(card.global_position):
		print("Card discarded outside board")
		#card.consume() # Later
		return

	if dropped_on_cards.is_empty(): return


	var target_card = dropped_on_cards[0] # Interact with the first overlapped card
	process_stack(card, target_card)


func _on_card_action_complete(card: Card):
	if game_over: return
	
	if card == gardener_card and card.card_properties.has("current_target"):
		var target = card.card_properties.current_target as Card
		if target and is_instance_valid(target):
			var action = card.card_properties.get("current_action", "")
			
			match action:
				"grow":
					# Check if energy was applied and advance stage
					if target.is_growable_plant() and target.card_properties.has("applied_energy"):
						target.card_properties.erase("applied_energy") # Consume energy visually
						var current_stage = CardDefs.get_property(target.card_type, "growth_stage", 0)
						var next_stage_type = CardDefs.CardType.NONE
						# Define growth path
						if target.card_type == CardDefs.CardType.SUNPETAL_SEED:
							next_stage_type = CardDefs.CardType.SUNPETAL_SPROUT
						elif target.card_type == CardDefs.CardType.SUNPETAL_SPROUT:
							next_stage_type = CardDefs.CardType.SUNPETAL_MATURE
						
						if next_stage_type != CardDefs.CardType.NONE:
							print("Plant grew to ", CardDefs.get_label(next_stage_type))
							var pos = target.global_position
							target.consume(false) # Remove old stage
							spawn_card(next_stage_type, pos)
						else:
							print("Plant reached max growth or invalid state.")
					else:
						print("Growth action finished, but conditions not met?")
				
				"harvest":
					if target.is_harvestable_plant():
						var yielded_resource = CardDefs.get_property(target.card_type, "yields", CardDefs.CardType.NONE)
						if yielded_resource != CardDefs.CardType.NONE:
							spawn_card(yielded_resource, target.global_position + Vector2(randf_range(60, 80), randf_range(-20, 20)))
							# HACK: could also revert it?
							target.consume(true)
						else:
							print("Plant is harvestable but yields nothing?")
					else:
						print("Harvest action finished, but target wasn't harvestable?")

				"craft":
					var recipe_card = card.card_properties.get("recipe_card_ref", null)
					if recipe_card and is_instance_valid(recipe_card):
						var recipe_type = recipe_card.card_type
						var product_type = CardDefs.CardType.NONE
						# Define recipe outcomes
						if recipe_type == CardDefs.CardType.CONCENTRATED_DEW_RECIPE:
							product_type = CardDefs.CardType.CONCENTRATED_DEW
						elif recipe_type == CardDefs.CardType.AEVUM_BLOOM_RECIPE:
							product_type = CardDefs.CardType.AEVUM_BLOOM

						if product_type != CardDefs.CardType.NONE:
							var pos = recipe_card.global_position
							recipe_card.consume(false) # Consume recipe card
							var product = spawn_card(product_type, pos + Vector2(0, 20))
							if product_type == CardDefs.CardType.AEVUM_BLOOM:
								win_game() # WIN CONDITION MET!
						else:
							print("Crafting finished, but no valid product for recipe?")
					else:
						print("Crafting finished, but recipe card missing?")

		# Clear Gardener's task info
		card.card_properties.erase("current_target")
		card.card_properties.erase("current_action")
		card.card_properties.erase("recipe_card_ref")

	# Make gardener available again if they were the one working
	if card == gardener_card:
		card.is_working = false # Explicitly free
		card.sprite.modulate = Color.WHITE 


func _on_card_destroyed(card: Card):
	if card in active_cards:
		active_cards.erase(card)
	if card == gardener_card:
		if not game_over: lose_game("The Gardener is gone!")
		gardener_card = null
	if card == chrono_crystal_card:
		if not game_over: lose_game("The Chrono Crystal shattered!")
		chrono_crystal_card = null


func _on_crystal_drain_timer_timeout():
	if game_over or not chrono_crystal_card: return
	current_crystal_energy -= crystal_drain_rate
	chrono_crystal_card.card_properties.energy = current_crystal_energy
	chrono_crystal_card.update_label()
	ui.update_crystal_energy(current_crystal_energy)

	if current_crystal_energy <= 0:
		lose_game("Chrono Crystal depleted!")

func _on_gardener_recharge_timer_timeout():
	if game_over or not gardener_card: return

	# Recharge only if gardener is IDLE
	if not gardener_card.is_working and not gardener_card.is_dragging:
		current_gardener_energy = min(max_gardener_energy, current_gardener_energy + gardener_recharge_rate)
		gardener_card.card_properties.energy = current_gardener_energy
		gardener_card.update_label()
		ui.update_gardener_energy(current_gardener_energy, max_gardener_energy)

func _on_energy_spawn_timer_timeout():
	if game_over: return
	# Spawn basic energy, maybe chance for Fleeting Moment
	var type_to_spawn = CardDefs.CardType.PAST_DEW
	if randf() < 0.2: # 20% chance for Fleeting Moment
		type_to_spawn = CardDefs.CardType.FLEETING_MOMENT

	var spawn_pos = Vector2(randf_range(board_size.position.x, board_size.end.x),
						   randf_range(board_size.position.y, board_size.end.y))
	# empty space check (can be better)
	var overlapping = get_tree().call_group.bind("cards", "get_overlapping_cards") # ... - check specific pos
	spawn_card(type_to_spawn, spawn_pos)


# func _on_anomaly_check_timer_timeout():
# 	if game_over: return
#   # Add logic here later to spawn Temporal Weeds based on certain conditions


func can_afford_action(cost: float) -> bool:
	if gardener_card and current_gardener_energy >= cost:
		return true
	print("Not enough Gardener Energy!")
	# TODO: Add visual feedback like shaking the gardener card
	return false

func spend_gardener_energy(cost: float):
	if gardener_card:
		current_gardener_energy -= cost
		gardener_card.card_properties.energy = current_gardener_energy
		gardener_card.update_label()
		ui.update_gardener_energy(current_gardener_energy, max_gardener_energy)

func process_stack(card_a: Card, card_b: Card):
	""" Processes interactions when card_a is dropped ONTO card_b. """
	if card_a.is_working or card_b.is_working: return # Ignore stacks with busy cards

	var type_a = card_a.card_type
	var type_b = card_b.card_type


	if type_a == CardDefs.CardType.GARDENER:
		var cost = gardener_action_cost # Base cost

		match type_b:
			# Harvesting Mature Plant
			CardDefs.CardType.SUNPETAL_MATURE:
				if can_afford_action(cost):
					print("Gardener starts harvesting ", CardDefs.get_label(type_b))
					spend_gardener_energy(cost)
					card_a.start_action_timer(harvest_time)
					card_a.card_properties.current_target = card_b
					card_a.card_properties.current_action = "harvest"
					card_a.global_position = card_b.global_position + Vector2(0, -40) # Stick visually

			# Tending Plant (Requires Energy Card also present - check overlaps)
			CardDefs.CardType.SUNPETAL_SEED, CardDefs.CardType.SUNPETAL_SPROUT:
				# Check if an energy card is also overlapping card_b
				var energy_card = find_overlapping_card_type(card_b, [CardDefs.CardType.PAST_DEW, CardDefs.CardType.FLEETING_MOMENT])
				if energy_card:
					if can_afford_action(cost):
						print("Gardener starts tending ", CardDefs.get_label(type_b), " with ", CardDefs.get_label(energy_card.card_type))
						spend_gardener_energy(cost)
						card_a.start_action_timer(plant_growth_time) # Use base time, energy type might modify later
						card_a.card_properties.current_target = card_b
						card_a.card_properties.current_action = "grow"
						card_b.card_properties.applied_energy = true # Mark plant as energized for completion check
						# Consume the energy card immediately
						energy_card.consume(true)
						card_a.global_position = card_b.global_position + Vector2(0, -40)
				else:
					print("Need an energy card (Past Dew/Fleeting Moment) on the plant to tend it.")
				
			# Crafting (Acting on Recipe Card)
			CardDefs.CardType.CONCENTRATED_DEW_RECIPE, CardDefs.CardType.AEVUM_BLOOM_RECIPE:
				if can_afford_action(cost):
					print("Gardener starts crafting ", CardDefs.get_label(type_b))
					spend_gardener_energy(cost)
					card_a.start_action_timer(craft_time)
					card_a.card_properties.current_target = card_b # Target is recipe
					card_a.card_properties.current_action = "craft"
					card_a.card_properties.recipe_card_ref = card_b # Store ref to consume later
					card_a.global_position = card_b.global_position + Vector2(0, -40)



	elif (type_a == CardDefs.CardType.PAST_DEW and type_b == CardDefs.CardType.PAST_DEW):
		print("Recipe: Concentrated Dew")
		spawn_card(CardDefs.CardType.CONCENTRATED_DEW_RECIPE, (card_a.global_position + card_b.global_position) / 2)
		card_a.consume(false)
		card_b.consume(false)

	elif (type_a == CardDefs.CardType.SUNPETAL_MATURE and type_b == CardDefs.CardType.CONCENTRATED_DEW) or \
		(type_a == CardDefs.CardType.CONCENTRATED_DEW and type_b == CardDefs.CardType.SUNPETAL_MATURE):
		print("Recipe: Aevum Bloom")
		spawn_card(CardDefs.CardType.AEVUM_BLOOM_RECIPE, (card_a.global_position + card_b.global_position) / 2)
		card_a.consume(false)
		card_b.consume(false)


	elif type_a == CardDefs.CardType.SUNPETAL_SEED and type_b == CardDefs.CardType.SEEDLING_PATCH:
		print("Planted Sunpetal Seed")
		# Simplest: Seed just transforms the patch? Or Seed stays on top? Let's keep Seed on top.
		# For this model, just placing seed on patch is enough, growth needs Gardener + Energy stack.
		card_a.global_position = card_b.global_position + Vector2(0, -10) # Snap visually

func find_overlapping_card_type(target_card: Card, types_to_find: Array[CardDefs.CardType]) -> Card:
	var overlaps = target_card.get_overlapping_cards()
	for card in overlaps:
		if card.card_type in types_to_find:
			return card
	return null

func win_game():
	if game_over: return
	print("Game Won!")
	game_over = true
	ui.show_win_message()
	# Stop timers
	crystal_drain_timer.stop()
	gardener_recharge_timer.stop()
	energy_spawn_timer.stop()
	#anomaly_check_timer.stop() # Stop if using

func lose_game(reason: String):
	if game_over: return # Prevent multiple triggers
	print("Game Lost! Reason: ", reason)
	game_over = true
	ui.show_lose_message(reason)
	# Stop timers
	crystal_drain_timer.stop()
	gardener_recharge_timer.stop()
	energy_spawn_timer.stop()
	#anomaly_check_timer.stop() # Stop if using
	# Make gardener visually 'spent' if they exist
	if gardener_card:
		gardener_card.is_working = true # Prevent further actions
		gardener_card.sprite.modulate = Color(0.5, 0.5, 0.5) # Grey tint

func _process(delta):
	# Allow restarting
	if Input.is_action_just_pressed("ui_accept"): # Enter key
		if game_over:
			print("Restarting game...")
			start_game()
