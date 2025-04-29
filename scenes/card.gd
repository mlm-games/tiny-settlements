# Card.gd - Complete rewrite
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

@onready var sprite: Sprite2D = $Sprite2D
@onready var label: Label = $Label
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

func set_card_type(new_type: CardDefs.CardType):
	card_type = new_type
	if CardDefs.CARD_PROPERTIES.has(card_type):
		card_properties = CardDefs.CARD_PROPERTIES[card_type].duplicate()

	# Setup based on type
	var passive_interval = CardDefs.get_property(card_type, "passive_interval", 0.0)
	if passive_interval > 0.0:
		current_passive_action_interval = passive_interval

	if card_type == CardDefs.CardType.MATURE_VINE:
		needs_pollination = true

	if label:
		update_label()

func update_label():
	if not label: return
	var base_label = CardDefs.get_label(card_type)
	var extra_info = ""

	# Add dynamic info based on state
	match card_type:
		CardDefs.CardType.GARDENER:
			if card_properties.has("focus"):
				extra_info = "\nFocus: %d%%" % int(card_properties.focus / card_properties.max_focus * 100.0)
		CardDefs.CardType.MATURE_VINE:
			if needs_pollination and not is_pollinated: 
				extra_info = "\n(Needs Pollination)"
			elif is_pollinated: 
				extra_info = "\n(Pollinated)"
		CardDefs.CardType.BASIC_FUNGI, CardDefs.CardType.SYMBIOTIC_ALGAE, CardDefs.CardType.GRAZING_SLUG:
			if not passive_action_timer.is_stopped():
				extra_info = "\n(%.1fs)" % passive_action_timer.time_left

	label.text = base_label + extra_info

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
