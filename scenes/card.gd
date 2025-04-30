class_name Card extends Area2D

signal drag_started(card)
signal drag_ended(card, dropped_on_cards)
signal action_complete(card)
signal passive_action_ready(card, action_type)
signal self_destruct(card)

@export var card_type : CardDefs.CardType = CardDefs.CardType.NONE : set = set_card_type
var card_properties : Dictionary = {}

# Dynamic State
var is_planted: bool = false
var needs_pollination: bool = false
var is_pollinated: bool = false
var current_passive_action_interval: float = 0.0

var is_dragging: bool = false
var is_working: bool = false
var drag_offset: Vector2 = Vector2.ZERO

var stacked_on: Card = null
var stacked_cards: Array[Card] = []

var light_effect: PointLight2D = null
var card_image: Texture2D = null
var interaction_indicator: Node2D = null

@onready var image_rect: TextureRect = $ImageRect
@onready var title_label: Label = $TitleLabel
@onready var status_label: Label = $StatusLabel

@onready var sprite: Sprite2D = $Sprite2D
@onready var label: Label = $TitleLabel
@onready var collision_shape: CollisionShape2D = $CollisionShape2D
@onready var animation_player: AnimationPlayer = $AnimationPlayer
@onready var action_timer: Timer = $ActionTimer
@onready var passive_action_timer: Timer = $PassiveActionTimer
@onready var lifespan_timer: Timer = $LifespanTimer

func _ready():
	action_timer.timeout.connect(_on_action_timer_timeout)
	passive_action_timer.timeout.connect(_on_passive_action_timer_timeout)
	lifespan_timer.timeout.connect(_on_lifespan_timer_timeout)

	# Spawn animation
	scale = Vector2.ZERO
	var tween = create_tween()
	tween.tween_property(self, "scale", Vector2.ONE, 0.2).set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)

	set_card_type(card_type)
	setup_card_visuals()
	
	if card_type in [CardDefs.CardType.LUMINA_CRYSTAL, 
					CardDefs.CardType.MATURE_FLUTTERWING, 
					CardDefs.CardType.GENESIS_BLOOM]:
		add_light_effect()
	
	setup_interaction_indicator()

func add_light_effect():
	light_effect = PointLight2D.new()
	light_effect.texture = preload("res://assets/soft_light.png")
	light_effect.energy = 0.7
	light_effect.texture_scale = 0.5
	
	# Different colors for different card types
	match card_type:
		CardDefs.CardType.LUMINA_CRYSTAL:
			light_effect.color = Color(0.5, 0.8, 1.0, 0.7)
		CardDefs.CardType.MATURE_FLUTTERWING:
			light_effect.color = Color(0.8, 0.6, 1.0, 0.5)
		CardDefs.CardType.GENESIS_BLOOM:
			light_effect.color = Color(0.7, 1.0, 0.8, 0.8)
			light_effect.energy = 1.0
			light_effect.texture_scale = 1.0
	
	add_child(light_effect)

func set_card_type(new_type: CardDefs.CardType):
	card_type = new_type
	if CardDefs.CARD_PROPERTIES.has(card_type):
		card_properties = CardDefs.CARD_PROPERTIES[card_type].duplicate()
	
	# Load the card image if specified
	if card_properties.has("image"):
		var image_path = card_properties.get("image", "")
		if image_path != "":
			card_image = load(image_path)
	
	var passive_interval = CardDefs.get_property(card_type, "passive_interval", 0.0)
	if passive_interval > 0.0:
		current_passive_action_interval = passive_interval

	if card_type == CardDefs.CardType.MATURE_VINE:
		needs_pollination = true

	if image_rect and title_label and status_label:
		setup_card_visuals()

func setup_card_visuals():
	if image_rect and card_image:
		image_rect.texture = card_image
	
	if title_label:
		title_label.text = CardDefs.get_label(card_type)
	
	update_label()

func update_label():
	if not status_label: return
	var status_text = ""

	match card_type:
		CardDefs.CardType.GARDENER:
			if card_properties.has("focus"):
				status_text = "Focus: %d%%" % int(card_properties.focus / card_properties.max_focus * 100.0)
		CardDefs.CardType.MATURE_VINE:
			if needs_pollination and not is_pollinated: 
				status_text = "(Needs Pollination)"
			elif is_pollinated: 
				status_text = "(Pollinated)"
		CardDefs.CardType.BASIC_FUNGI, CardDefs.CardType.SYMBIOTIC_ALGAE, CardDefs.CardType.GRAZING_SLUG:
			if not passive_action_timer.is_stopped():
				status_text = "(%.1fs)" % passive_action_timer.time_left
	
	status_label.text = status_text
	
	# Auto-adjust font size if text doesn't fit
	var original_size = 14 # Default font size
	status_label.add_theme_font_size_override("font_size", original_size)
	
	# Wait for the label to update its size
	await get_tree().process_frame
	
	# Check if text is too large and adjust if needed
	if status_label.get_line_count() > 2:
		var new_size = original_size * 0.8
		status_label.add_theme_font_size_override("font_size", new_size)

func _input_event(viewport, event, shape_idx):
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		if event.pressed and not is_working:
			# Allow dragging any card that isn't busy
			is_dragging = true
			drag_offset = get_global_mouse_position() - global_position
			z_index = 10
			emit_signal("drag_started", self)
			get_viewport().set_input_as_handled()
			play_feedback_animation("pickup")
		elif not event.pressed and is_dragging:
			is_dragging = false
			z_index = 0
			emit_signal("drag_ended", self, get_overlapping_cards())
			get_viewport().set_input_as_handled()
			play_feedback_animation("drop")

func _process(delta):
	if is_dragging:
		global_position = get_global_mouse_position() - drag_offset
	
	if not is_dragging and not is_working:
		update_stacking_relationships()

func update_stacking_relationships():
	# Clear previous relationships
	if stacked_on != null:
		stacked_on.stacked_cards.erase(self)
		stacked_on = null
	
	# Check for cards below this one
	var overlaps = get_overlapping_cards()
	for card in overlaps:
		if not is_instance_valid(card):
			continue
			
		# Check if this card is visually on top of another card
		if global_position.y < card.global_position.y + 10 and \
		   global_position.y > card.global_position.y - 30:
			stacked_on = card
			if not card.stacked_cards.has(self):
				card.stacked_cards.append(self)
			break

func start_gardener_action(duration: float):
	if not is_working:
		is_working = true
		action_timer.wait_time = duration
		action_timer.start()
		sprite.modulate = Color(0.9, 0.9, 0.9)

func _on_action_timer_timeout():
	is_working = false
	sprite.modulate = Color.WHITE
	emit_signal("action_complete", self)

func start_passive_timer(action_type: String, custom_interval: float = -1.0):
	if passive_action_timer.is_stopped():
		var interval = custom_interval if custom_interval > 0.0 else current_passive_action_interval
		if interval > 0.0:
			passive_action_timer.editor_description = action_type
			passive_action_timer.wait_time = interval
			passive_action_timer.start()
			update_label()

func stop_passive_timer():
	passive_action_timer.stop()
	update_label()

func _on_passive_action_timer_timeout():
	var action_type = passive_action_timer.editor_description
	emit_signal("passive_action_ready", self, action_type)

func _on_lifespan_timer_timeout():
	emit_signal("self_destruct", self)
	consume(true)

func consume(play_anim: bool = true):
	# Stop timers before consuming
	action_timer.stop()
	passive_action_timer.stop()
	lifespan_timer.stop()

	if play_anim:
		is_working = true
		play_feedback_animation("consumed")
		await get_tree().create_timer(0.2).timeout
	queue_free()

func get_overlapping_cards() -> Array[Card]:
	var overlapping_cards : Array[Card] = []
	var areas = get_overlapping_areas()
	for area in areas:
		if area is Card and area != self:
			overlapping_cards.append(area)
	return overlapping_cards

func play_feedback_animation(anim_name: String):
	if not animation_player: return
	
	match anim_name:
		"pickup":
			if not animation_player.has_animation("pickup"): 
				_create_simple_anim("pickup", "scale", Vector2(1.0, 1.0), Vector2(1.1, 1.1), 0.1)
			animation_player.play("pickup")
		"drop":
			if not animation_player.has_animation("drop"): 
				_create_simple_anim("drop", "scale", Vector2(1.1, 1.1), Vector2(1.0, 1.0), 0.1)
			animation_player.play("drop")
		"working":
			if not animation_player.has_animation("working"):
				# Use existing working animation
				pass
			animation_player.play("working")
		"complete":
			if not animation_player.has_animation("complete"):
				# Use existing complete animation
				pass
			animation_player.play("complete")
		"consumed":
			if not animation_player.has_animation("consumed"): 
				_create_simple_anim("consumed", "modulate:a", 1.0, 0.0, 0.2)
			animation_player.play("consumed")
		"passive_pulse":
			if not animation_player.has_animation("passive_pulse"): 
				_create_simple_anim("passive_pulse", "scale", Vector2(1.0, 1.0), Vector2(1.05, 1.05), 0.3, true)
			animation_player.play("passive_pulse")
		"error_pulse":
			if not animation_player.has_animation("error_pulse"):
				# Create a red flash animation for errors
				var anim = Animation.new()
				anim.add_track(Animation.TYPE_VALUE)
				anim.track_set_path(0, "modulate")
				anim.length = 0.4
				anim.track_insert_key(0, 0.0, Color.WHITE)
				anim.track_insert_key(0, 0.1, Color(1.0, 0.5, 0.5, 1.0))
				anim.track_insert_key(0, 0.4, Color.WHITE)
				animation_player.add_animation("error_pulse", anim)
			animation_player.play("error_pulse")

# Helper to create simple animations
func _create_simple_anim(name: String, track_path: String, start_val, end_val, duration: float, bounce_back: bool = false):
	var anim = Animation.new()
	anim.add_track(Animation.TYPE_VALUE)
	anim.track_set_path(0, track_path)
	anim.length = duration
	anim.track_insert_key(0, 0.0, start_val)
	
	if bounce_back:
		anim.track_insert_key(0, duration * 0.5, end_val)
		anim.track_insert_key(0, duration, start_val)
	else:
		anim.track_insert_key(0, duration, end_val)
		
	animation_player.get_animation_library("").add_animation(name, anim)

# Reset any stuck state
func reset_interaction_state():
	is_working = false
	is_dragging = false
	sprite.modulate = Color.WHITE
	z_index = 0
	
	# Stop any running timers
	if action_timer.time_left > 0:
		action_timer.stop()
	
	# Clear any temporary action data
	if card_properties.has("gardener_action_data"):
		card_properties.erase("gardener_action_data")


func setup_interaction_indicator():
	interaction_indicator = Node2D.new()
	interaction_indicator.name = "InteractionIndicator"
	add_child(interaction_indicator)
	
	# Create the indicator visuals
	var indicator_sprite = Sprite2D.new()
	indicator_sprite.texture = preload("res://assets/interaction_indicator.png")
	indicator_sprite.position = Vector2(0, -50)
	indicator_sprite.scale = Vector2(0.5, 0.5)
	indicator_sprite.visible = false
	interaction_indicator.add_child(indicator_sprite)

func show_interaction_hint(hint_type: String):
	var indicator_sprite = interaction_indicator.get_node("Sprite2D")
	if not indicator_sprite:
		return
	
	indicator_sprite.visible = true
	
	# Different colors/animations based on hint type
	match hint_type:
		"needs_gardener":
			indicator_sprite.modulate = Color(0.9, 0.8, 0.3)
			_animate_indicator_pulse(indicator_sprite)
		"needs_nutrient":
			indicator_sprite.modulate = Color(0.3, 0.9, 0.5)
			_animate_indicator_pulse(indicator_sprite)
		"needs_pollination":
			indicator_sprite.modulate = Color(0.8, 0.5, 0.9)
			_animate_indicator_pulse(indicator_sprite)
		"error":
			indicator_sprite.modulate = Color(0.9, 0.3, 0.3)
			_animate_indicator_shake(indicator_sprite)
	
	# Auto-hide after a few seconds
	await get_tree().create_timer(2.0).timeout
	hide_interaction_hint()

func hide_interaction_hint():
	var indicator_sprite = interaction_indicator.get_node("Sprite2D")
	if indicator_sprite:
		indicator_sprite.visible = false

func _animate_indicator_pulse(sprite: Sprite2D):
	var tween = create_tween()
	tween.tween_property(sprite, "scale", Vector2(0.6, 0.6), 0.5)
	tween.tween_property(sprite, "scale", Vector2(0.5, 0.5), 0.5)
	tween.set_loops(2)

func _animate_indicator_shake(sprite: Sprite2D):
	var tween = create_tween()
	tween.tween_property(sprite, "position", Vector2(5, -50), 0.1)
	tween.tween_property(sprite, "position", Vector2(-5, -50), 0.1)
	tween.tween_property(sprite, "position", Vector2(0, -50), 0.1)
	tween.set_loops(2)
