extends Node

const CardScene = preload("uid://din71ct0bjm3d")
const CardDefs = preload("uid://drx32as7xbxxr")

var current_day: int = 1
var game_over: bool = false
var villager_card: Card = null # Reference to the single villager (later for game over checks)

@onready var game_board: Node2D = %GameBoard
@onready var day_timer: Timer = %DayTimer
@onready var hunger_timer: Timer = %HungerTimer
@onready var ui: CanvasLayer = %UI

var active_cards: Array[Card] = []

@export var gather_time: float = 2.0
@export var craft_time: float = 4.0
@export var hunger_per_tick: float = 2.0
@export var hunger_per_day: float = 20.0
@export var board_size = Rect2(50, 100, 800, 500) # cards can exist only here


func _ready():
	day_timer.timeout.connect(_on_day_timer_timeout)
	hunger_timer.timeout.connect(_on_hunger_timer_timeout)
	
	start_game()


func start_game():
	# If restarting
	for card in get_tree().get_nodes_in_group("cards"):
		card.queue_free()
	active_cards.clear()
	game_over = false
	current_day = 1
	villager_card = null
	
	spawn_card(CardDefs.CardType.VILLAGER, Vector2(150, 300))
	spawn_card(CardDefs.CardType.TREE, Vector2(300, 250))
	spawn_card(CardDefs.CardType.BERRY_BUSH, Vector2(450, 300))
	spawn_card(CardDefs.CardType.ROCK_PILE, Vector2(300, 400))
	
	ui.update_day(current_day)
	ui.update_time(day_timer.wait_time)
	if villager_card:
		ui.update_hunger(villager_card.card_properties.hunger, villager_card.card_properties.max_hunger)
	ui.clear_status()


func spawn_card(type: CardDefs.CardType, position: Vector2) -> Card:
	#if game_over: return null
	
	var new_card : Card = CardScene.instantiate()
	game_board.add_child(new_card)
	new_card.card_type = type
	
	
	if type == CardDefs.CardType.VILLAGER:
		if villager_card == null: # Ensure only one villager for jam (to prevent scope creep)
			villager_card = new_card
			new_card.card_properties.hunger = new_card.card_properties.initial_hunger
			new_card.update_label() 
		else:
			print("Error: Tried to spawn second villager.")
			new_card.queue_free()
			return null
	
	new_card.global_position = position
	#game_board.add_child(new_card)
	active_cards.append(new_card)
	new_card.add_to_group("cards")
	
	new_card.drag_started.connect(_on_card_drag_started)
	new_card.drag_ended.connect(_on_card_drag_ended)
	new_card.action_complete.connect(_on_card_action_complete)
	new_card.tree_exiting.connect(_on_card_destroyed.bind(new_card))
	
	return new_card


func _on_card_drag_started(card: Card):
	# Maybe dim other cards slightly? (for visual feedback.) Template stuff mostly for later
	pass

func _on_card_drag_ended(card: Card, dropped_on_cards: Array[Card]):
	if game_over: return
	
	if not board_size.has_point(card.global_position):
		print("Card discarded outside board")
		card.consume()
		return
	
	if dropped_on_cards.is_empty():
		# If dropped on empty spaces, do nothing special
		return
	
	
	# Simple case: Card A dropped onto Card B (first in overlapp list)
	if dropped_on_cards.size() > 0:
		var target_card = dropped_on_cards[0]
		process_stack(card, target_card)
	
	# More complex: Check combinations under the Villager
	if card.card_type == CardDefs.CardType.VILLAGER:
		check_villager_crafting_recipes(card, dropped_on_cards)


func _on_card_action_complete(card: Card):
	if game_over: return
	
	# Check what action was completed based on card type and maybe context and then the RESULT of the action happens (ex: spawning resources)
	
	if card == villager_card and card.card_properties.has("current_target"):
		var target = card.card_properties.current_target as Card
		if target and is_instance_valid(target):
			match target.card_type:
				CardDefs.CardType.TREE, CardDefs.CardType.ROCK_PILE, CardDefs.CardType.BERRY_BUSH:
					var yielded_resource = CardDefs.CARD_PROPERTIES[target.card_type].yields
					spawn_card(yielded_resource, target.global_position + Vector2(randf_range(60, 80), randf_range(-20, 20)))
					if target.decrease_durability(): # Decrease durability, consume if needed
						pass # Target consumed itself

		card.card_properties.erase("current_target")

	elif card == villager_card and card.card_properties.has("crafting_recipe"):
		var recipe_type = card.card_properties.crafting_recipe
		var recipe_card = card.card_properties.get("recipe_card_ref", null) # Get reference if using recipe cards
		var position = card.global_position 

		if recipe_card and is_instance_valid(recipe_card):
			position = recipe_card.global_position
			recipe_card.consume(false) 

		match recipe_type:
			CardDefs.CardType.CAMPFIRE:
				spawn_card(CardDefs.CardType.CAMPFIRE, position + Vector2(0, 20))
			CardDefs.CardType.PLANK:
				spawn_card(CardDefs.CardType.PLANK, position + Vector2(0, 20))
			CardDefs.CardType.STURDY_HUT:
				var hut = spawn_card(CardDefs.CardType.STURDY_HUT, position + Vector2(0, 20))
			
				if hut:
					win_game()

		card.card_properties.erase("crafting_recipe")
		card.card_properties.erase("recipe_card_ref")

	# Make villager available again if they were the one working
	if card == villager_card:
		card.is_working = false
		card.sprite.modulate = Color.WHITE 


func _on_card_destroyed(card: Card):
	if card in active_cards:
		active_cards.erase(card)
	if card == villager_card:
		villager_card = null
		if not game_over:
			lose_game("The Villager is gone!")


func _on_day_timer_timeout():
	if game_over: return
	current_day += 1
	ui.update_day(current_day)

	if villager_card:
		villager_card.card_properties.hunger += hunger_per_day
		check_hunger() 

	if randf() < 0.3: # 30% chance each day, need to make it a ver?
		var res_type = [CardDefs.CardType.TREE, CardDefs.CardType.ROCK_PILE, CardDefs.CardType.BERRY_BUSH].pick_random()
		
		var spawn_pos = Vector2(randf_range(board_size.position.x, board_size.end.x),
							   randf_range(board_size.position.y, board_size.end.y))
		spawn_card(res_type, spawn_pos)


func _on_hunger_timer_timeout():
	if game_over or not villager_card: return

	villager_card.card_properties.hunger += hunger_per_tick
	check_hunger() 


func process_stack(card_a: Card, card_b: Card):
	"""Processes interactions when card_a is dropped ONTO card_b."""
	if card_a.is_working or card_b.is_working: return

	var type_a = card_a.card_type
	var type_b = card_b.card_type


	if type_a == CardDefs.CardType.VILLAGER:
		match type_b:
			CardDefs.CardType.TREE, CardDefs.CardType.ROCK_PILE, CardDefs.CardType.BERRY_BUSH: # Gatherable
				if not card_a.is_working:
					print("Villager starts gathering from ", CardDefs.get_label(type_b))
					card_a.start_action_timer(gather_time)
				
					card_a.card_properties.current_target = card_b
					
					card_a.global_position = card_b.global_position + Vector2(0, -40) # for sticky effect
				
			# Eating Food
			CardDefs.CardType.BERRIES:
				print("Villager eats Berries")
				var food_value = CardDefs.CARD_PROPERTIES[type_b].food_value
				card_a.card_properties.hunger = max(0.0, card_a.card_properties.hunger - food_value)
				card_a.update_label()
				card_b.consume()
				
			# Crafting (Initiated by Villager on Recipe Card (for now))
			CardDefs.CardType.CAMPFIRE_RECIPE, CardDefs.CardType.PLANK_RECIPE, CardDefs.CardType.STURDY_HUT_RECIPE:
				if not card_a.is_working:
					print("Villager starts crafting ", CardDefs.get_label(type_b))
					card_a.start_action_timer(craft_time)
					card_a.card_properties.crafting_recipe = CardDefs.CARD_PROPERTIES[type_b]["produces"] # Assumes recipe definition includes output type
					card_a.card_properties.recipe_card_ref = card_b # Store ref to consume later
					
					card_a.global_position = card_b.global_position + Vector2(0, -40)
			
		
		
	# This happens *before* villager interaction for these recipes
	elif (type_a == CardDefs.CardType.WOOD and type_b == CardDefs.CardType.STONE) or \
		 (type_a == CardDefs.CardType.STONE and type_b == CardDefs.CardType.WOOD):
		print("Recipe: Campfire")
		spawn_card(CardDefs.CardType.CAMPFIRE_RECIPE, (card_a.global_position + card_b.global_position) / 2)
		card_a.consume(false)
		card_b.consume(false)
		
	elif type_a == CardDefs.CardType.WOOD and type_b == CardDefs.CardType.WOOD: # Example: 1 Wood for Plank Recipe
		print("Recipe: Plank")
		# For the jam, let's assume dropping Wood onto Wood creates the recipe card
		# A better system might require dropping Wood onto a specific 'Workbench' card later
		spawn_card(CardDefs.CardType.PLANK_RECIPE, (card_a.global_position + card_b.global_position) / 2)
		card_a.consume(false)
		card_b.consume(false) # Consume both woods for now? Let's say 1 wood = 1 plank recipe for a workbench for it to be more efficient
		
	elif (type_a == CardDefs.CardType.PLANK and type_b == CardDefs.CardType.STONE) or \
		(type_a == CardDefs.CardType.STONE and type_b == CardDefs.CardType.PLANK):
		print("Recipe: Sturdy Hut")
		spawn_card(CardDefs.CardType.STURDY_HUT_RECIPE, (card_a.global_position + card_b.global_position) / 2)
		card_a.consume(false)
		card_b.consume(false)
	
	if villager_card:
		ui.update_hunger(villager_card.card_properties.hunger, villager_card.card_properties.max_hunger)


func check_villager_crafting_recipes(villager: Card, overlapped_cards: Array[Card]):
	""" Checks if villager was dropped on a valid combo of items for crafting.
		This is an alternative/addition to recipe cards.
		For this simple example, we'll stick to recipe cards created above."""
	# Example of direct crafting check (if not using recipe cards):
	# var found_wood = null
	# var found_stone = null
	# for item in overlapped_cards:
	#     if item.card_type == CardDefs.CardType.WOOD: found_wood = item
	#     if item.card_type == CardDefs.CardType.STONE: found_stone = item
	#
	# if found_wood and found_stone and not villager.is_working:
	#     print("Villager starts crafting Campfire directly")
	#     villager.start_action_timer(craft_time)
	#     villager.card_properties.crafting_recipe = CardDefs.CardType.CAMPFIRE # Mark what's being crafted
	#     # Consume ingredients now or on completion? Let's do on completion via the recipe logic
	#     # We need to track the ingredients to consume them later
	#     villager.card_properties.ingredients = [found_wood, found_stone]
	#     # Stick villager
	#     villager.global_position = (found_wood.global_position + found_stone.global_position) / 2 + Vector2(0, -40)
	#     # Consume ingredients (Alternative: Consume now)
	#     # found_wood.consume(false)
	#     # found_stone.consume(false)
	#     return # Stop further checks if a recipe matched
	pass # Sticking to recipe cards for now


func check_hunger():
	if not villager_card: return

	var hunger = villager_card.card_properties.hunger
	var max_hunger = villager_card.card_properties.max_hunger
	villager_card.update_label() # Update label text
	ui.update_hunger(hunger, max_hunger)

	if hunger >= max_hunger:
	
		lose_game("Villager starved!")

func win_game():
	print("Game Won!")
	game_over = true
	ui.show_win_message()
	day_timer.stop()
	hunger_timer.stop()

func lose_game(reason: String):
	if game_over: return
	print("Game Lost! Reason: ", reason)
	game_over = true
	ui.show_lose_message(reason)
	
	day_timer.stop()
	hunger_timer.stop()
	
	if villager_card:
		villager_card.is_working = true 
		villager_card.sprite.modulate = Color(0.5, 0.1, 0.1) 

func _process(delta):
	if game_over: return

	
	if day_timer.is_stopped() == false:
		ui.update_time(day_timer.time_left)

	# Allow restarting (simple implementation), #TODO: Add it to button as shortcut later
	if Input.is_action_just_pressed("ui_accept"):
		if game_over:
			print("Restarting game...")
			start_game()

# Placeholder for card definitions - adjust craft recipes in Main.gd
# Add these to CardDefinitions.gd or define directly in Main.gd if preferred
# This example assumes the recipe cards track what they produce implicitly via their type.
# You might add explicit properties to the recipe cards if needed.
#func _initialize_recipe_definitions():
	# Example of adding recipe data if not implicit in type
	#CardDefs.CARD_PROPERTIES[CardDefs.CardType.CAMPFIRE_RECIPE] = {"label": "Build Campfire\n(Need Villager)", "produces": CardDefs.CardType.CAMPFIRE}
	#CardDefs.CARD_PROPERTIES[CardDefs.CardType.PLANK_RECIPE] = {"label": "Make Plank\n(Need Villager)", "produces": CardDefs.CardType.PLANK}
	#CardDefs.CARD_PROPERTIES[CardDefs.CardType.STURDY_HUT_RECIPE] = {"label": "Build Hut\n(Need Villager)", "produces": CardDefs.CardType.STURDY_HUT}

# Call in _ready() if using _initialize_recipe_definitions
