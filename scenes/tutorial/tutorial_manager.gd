# TutorialManager.gd - Rewrite
extends Node
class_name TutorialManager

signal tutorial_complete
#FIXME: Currently the gardener recognises the substrates instead of planted spores when they overlap, so is harder to get it to act, but it works from the direction of the offset of the card
# Enum to track tutorial progress
enum State {
	OFF,
	INTRO,
	DRAG_INFO,
	FOCUS_INFO,
	PLANT_SEED_A, # Drag Seed to Substrate
	PLANT_SEED_B, # Drag Gardener to Seed
	PLANT_SEED_WAIT, # Wait for planting action
	APPLY_NUTRIENT_A, # Drag Nutrient near Plant
	APPLY_NUTRIENT_B, # Drag Gardener to Nutrient
	APPLY_NUTRIENT_WAIT, # Wait for applying action
	GROWTH_INFO, # Explain growth
	PASSIVE_INFO, # Explain Fungi production
	BIODIVERSITY_INFO, # Explain UI
	FINAL_MESSAGE
}

var current_state: State = State.OFF
var is_active: bool = false

# References needed from Main
var main_game: Node = null
var gardener_card_ref: Card = null
var initial_spore_ref: Card = null
var initial_substrate_ref: Card = null
var initial_nutrient_ref: Card = null
var grown_fungi_ref: Card = null

# UI Nodes
@onready var ui_layer: CanvasLayer = $TutorialUI
@onready var instruction_label: Label = $TutorialUI/MarginContainer/PanelContainer/InstructionLabel
@onready var highlight_rect: ColorRect = $TutorialUI/HighlightRect
@onready var end_timer: Timer = %EndTimer

func _ready():
	end_timer.timeout.connect(_on_end_timer_timeout)
	ui_layer.hide()
	highlight_rect.hide()

func start_tutorial(main_node: Node, gardener: Card, spore: Card, substrate: Card, nutrient: Card):
	if is_active: return # Already running
	
	print("Starting Tutorial")
	is_active = true
	main_game = main_node
	gardener_card_ref = gardener
	initial_spore_ref = spore
	initial_substrate_ref = substrate
	initial_nutrient_ref = nutrient
	current_state = State.INTRO
	
	# Connect signals from main game
	if main_game:
		if not main_game.tutorial_card_spawned.is_connected(on_main_card_spawned):
			main_game.tutorial_card_spawned.connect(on_main_card_spawned)
		if not main_game.tutorial_drag_ended.is_connected(on_main_drag_ended):
			main_game.tutorial_drag_ended.connect(on_main_drag_ended)
		if not main_game.tutorial_action_complete.is_connected(on_main_gardener_action_complete):
			main_game.tutorial_action_complete.connect(on_main_gardener_action_complete)
	
	ui_layer.show()
	update_tutorial_step()

func advance_tutorial():
	if not is_active: return
	
	# Increment state safely
	var next_state_index = current_state + 1
	if next_state_index <= State.FINAL_MESSAGE:
		current_state = next_state_index
		update_tutorial_step()
	else:
		end_tutorial() # Reached end

func update_tutorial_step():
	if not is_active: return
	highlight_rect.hide() # Hide highlight by default

	match current_state:
		State.INTRO:
			instruction_label.text = "Welcome to the Symbiotic Ecosystem! Your goal is to cultivate diverse life. Let's start with the basics."
			await get_tree().create_timer(5).timeout
			advance_tutorial()
			
		State.DRAG_INFO:
			instruction_label.text = "Everything is done by dragging cards. Try dragging the Nutrient Slime card around."
			if is_instance_valid(initial_nutrient_ref):
				position_highlight(initial_nutrient_ref)
			# Will advance when player drags the nutrient card
			
		State.FOCUS_INFO:
			instruction_label.text = "The 'Gardener' card is your tool. Actions cost Focus (top left UI). Focus recharges slowly when the Gardener is idle."
			if is_instance_valid(gardener_card_ref):
				position_highlight(gardener_card_ref)
			await get_tree().create_timer(5).timeout
			advance_tutorial()
			
		State.PLANT_SEED_A:
			instruction_label.text = "To grow life, plant seeds on substrate. Drag the 'Spore Pod' onto the 'Bio-Substrate'."
			if is_instance_valid(initial_spore_ref): 
				position_highlight(initial_spore_ref)
			if is_instance_valid(initial_substrate_ref): 
				position_highlight(initial_substrate_ref, true)
			# Will advance on correct drag end
			
		State.PLANT_SEED_B:
			instruction_label.text = "Good! Now, use the 'Gardener' to finalize planting. Drag the 'Gardener' onto the 'Spore Pod' (which is now on the substrate)."
			if is_instance_valid(gardener_card_ref): 
				position_highlight(gardener_card_ref)
			if is_instance_valid(initial_spore_ref): 
				position_highlight(initial_spore_ref, true)
			# Will advance on correct drag end
			
		State.PLANT_SEED_WAIT:
			instruction_label.text = "Planting initiated! Watch the Gardener work (it costs Focus). The action completes automatically."
			# Will advance when action complete signal is received
			
		State.APPLY_NUTRIENT_A:
			instruction_label.text = "Seeds need food to grow (sometimes). Drag the 'Nutrient Slime' near the planted 'Spore Pod'."
			if is_instance_valid(initial_nutrient_ref): 
				position_highlight(initial_nutrient_ref)
			if is_instance_valid(initial_spore_ref): 
				position_highlight(initial_spore_ref, true)
			# Will advance on correct drag end
			await get_tree().create_timer(10).timeout
			advance_tutorial()
			
		State.APPLY_NUTRIENT_B:
			instruction_label.text = "Now apply the nutrient (if needed). Drag the 'Gardener' onto the 'Nutrient Slime' (near the spore)."
			if is_instance_valid(gardener_card_ref): 
				position_highlight(gardener_card_ref)
			if is_instance_valid(initial_nutrient_ref): 
				position_highlight(initial_nutrient_ref, true)
			# Will advance on correct drag end
			await get_tree().create_timer(10).timeout
			advance_tutorial()
			
		State.APPLY_NUTRIENT_WAIT:
			instruction_label.text = "Applying nutrient... This also costs Focus."
			# Will advance when action complete signal is received
			await get_tree().create_timer(2).timeout
			advance_tutorial()
			
		State.GROWTH_INFO:
			instruction_label.text = "Success! Applying the right nutrient triggers growth. The Spore Pod should now grow into Basic Fungi."
			# Will advance when fungi card is spawned
			await get_tree().create_timer(3).timeout
			advance_tutorial()
			
		State.PASSIVE_INFO:
			instruction_label.text = "This Basic Fungi passively produces Processed Nutrients over time (see timer). Use these for more advanced life!"
			if is_instance_valid(grown_fungi_ref): 
				position_highlight(grown_fungi_ref)
			await get_tree().create_timer(3).timeout
			advance_tutorial()
			
		State.BIODIVERSITY_INFO:
			instruction_label.text = "Your 'Biodiversity' score (top left) increases as you cultivate new mature species. Discovering interactions is key!"
			await get_tree().create_timer(6).timeout
			advance_tutorial()
			
		State.FINAL_MESSAGE:
			instruction_label.text = "You've learned the basics! Experiment with new cards, discover symbiotic relationships, and grow your ecosystem. Good luck!"
			end_timer.start()

func _process(delta):
	# Allow skipping certain tutorial steps with Enter/Space
	if is_active and Input.is_action_just_pressed("ui_accept"):
		if current_state in [State.INTRO, State.DRAG_INFO, State.FOCUS_INFO, State.PASSIVE_INFO, State.BIODIVERSITY_INFO]:
			advance_tutorial()
	
	# Make highlight follow cards if they move
	if is_active and highlight_rect.visible:
		update_highlight_positions()

func update_highlight_positions():
	match current_state:
		State.DRAG_INFO:
			if is_instance_valid(initial_nutrient_ref):
				position_highlight(initial_nutrient_ref)
		State.FOCUS_INFO:
			if is_instance_valid(gardener_card_ref):
				position_highlight(gardener_card_ref)
		State.PLANT_SEED_A:
			if is_instance_valid(initial_spore_ref): 
				position_highlight(initial_spore_ref)
			if is_instance_valid(initial_substrate_ref): 
				position_highlight(initial_substrate_ref, true)
		State.PLANT_SEED_B:
			if is_instance_valid(gardener_card_ref): 
				position_highlight(gardener_card_ref)
			if is_instance_valid(initial_spore_ref): 
				position_highlight(initial_spore_ref, true)
		State.APPLY_NUTRIENT_A:
			if is_instance_valid(initial_nutrient_ref): 
				position_highlight(initial_nutrient_ref)
			if is_instance_valid(initial_spore_ref): 
				position_highlight(initial_spore_ref, true)
		State.APPLY_NUTRIENT_B:
			if is_instance_valid(gardener_card_ref): 
				position_highlight(gardener_card_ref)
			if is_instance_valid(initial_nutrient_ref): 
				position_highlight(initial_nutrient_ref, true)
		State.PASSIVE_INFO:
			if is_instance_valid(grown_fungi_ref): 
				position_highlight(grown_fungi_ref)

# --- Signal Handlers ---

func on_main_drag_ended(card_a: Card, card_b: Card):
	if not is_active: return
	if not is_instance_valid(card_a) or not is_instance_valid(card_b): return

	print("Tutorial received drag ended signal: ", CardDefs.get_label(card_a.card_type), " -> ", CardDefs.get_label(card_b.card_type))

	# Check if the correct drag happened for the current step
	match current_state:
		State.DRAG_INFO:
			if card_a == initial_nutrient_ref:
				advance_tutorial()
				
		State.PLANT_SEED_A:
			if card_a == initial_spore_ref and card_b == initial_substrate_ref:
				advance_tutorial()
				
		State.PLANT_SEED_B:
			if card_a == gardener_card_ref and card_b == initial_spore_ref:
				advance_tutorial() # Now wait for action
				
		State.APPLY_NUTRIENT_A:
			if card_a == initial_nutrient_ref:
				# Check if it's near the spore
				if is_instance_valid(initial_spore_ref) and card_a.global_position.distance_to(initial_spore_ref.global_position) < 100:
					advance_tutorial()
					
		State.APPLY_NUTRIENT_B:
			if card_a == gardener_card_ref and card_b == initial_nutrient_ref:
				# Check if nutrient is near spore
				if is_instance_valid(initial_spore_ref) and card_b.global_position.distance_to(initial_spore_ref.global_position) < 100:
					advance_tutorial() # Now wait for action

func on_main_gardener_action_complete(target_card: Card, action_data: Dictionary):
	if not is_active: return
	
	print("Tutorial received action complete: ", action_data.get("action", "unknown"))

	# Check if the completed action matches the waiting step
	var action = action_data.get("action", "")
	
	match current_state:
		State.PLANT_SEED_WAIT:
			if action == "plant" and target_card == initial_spore_ref:
				advance_tutorial()
				
		State.APPLY_NUTRIENT_WAIT:
			if action == "apply_nutrient" and target_card == initial_spore_ref:
				advance_tutorial()

func on_main_card_spawned(card: Card):
	if not is_active: return
	
	print("Tutorial received card spawned: ", CardDefs.get_label(card.card_type))

	# Check if the awaited growth happened
	if current_state == State.GROWTH_INFO and card.card_type == CardDefs.CardType.BASIC_FUNGI:
		grown_fungi_ref = card # Store reference
		advance_tutorial()

# --- UI Helpers ---

func position_highlight(target_node: Node2D, additive: bool = false):
	if not is_instance_valid(target_node):
		if not additive:
			highlight_rect.hide()
		return

	# Get size from collision shape or sprite
	var shape_node = target_node.get_node_or_null("CollisionShape2D")
	var target_size: Vector2
	
	if shape_node and shape_node.shape is RectangleShape2D:
		target_size = shape_node.shape.size * target_node.global_scale
	else:
		var sprite_node = target_node.get_node_or_null("Sprite2D")
		if sprite_node:
			target_size = sprite_node.get_rect().size * target_node.global_scale
		else:
			target_size = Vector2(60, 80) #
				# Add padding
			target_size += Vector2(10, 10)

	if additive and highlight_rect.visible:
		# For now, just reposition to the new target
		highlight_rect.global_position = target_node.global_position - target_size / 2.0
		highlight_rect.size = target_size
	else:
		highlight_rect.global_position = target_node.global_position - target_size / 2.0
		highlight_rect.size = target_size
		highlight_rect.show()

func end_tutorial():
	if not is_active: return
	
	print("Ending Tutorial")
	is_active = false
	current_state = State.OFF
	
	# Disconnect signals
	if main_game:
		if main_game.tutorial_card_spawned.is_connected(on_main_card_spawned):
			main_game.tutorial_card_spawned.disconnect(on_main_card_spawned)
		if main_game.tutorial_drag_ended.is_connected(on_main_drag_ended):
			main_game.tutorial_drag_ended.disconnect(on_main_drag_ended)
		if main_game.tutorial_action_complete.is_connected(on_main_gardener_action_complete):
			main_game.tutorial_action_complete.disconnect(on_main_gardener_action_complete)
	
	ui_layer.hide()
	highlight_rect.hide()
	emit_signal("tutorial_complete")

func _on_end_timer_timeout():
	end_tutorial()
