extends Area2D
class_name Card

const CardDefs = preload("uid://drx32as7xbxxr")

signal drag_started(card)
signal drag_ended(card, dropped_on_cards)
signal action_complete(card)
signal self_destruct(card) # For cards with lifespan

@export var card_type : CardDefs.CardType = CardDefs.CardType.NONE : set = set_card_type
var card_properties : Dictionary = {}

var is_dragging: bool = false
var is_working: bool = false # Is this card busy with a timed action (e.g., Gardener working)
var drag_offset: Vector2 = Vector2.ZERO

@onready var sprite: Sprite2D = $Sprite2D
@onready var label: Label = $Label
@onready var collision_shape: CollisionShape2D = $CollisionShape2D
@onready var animation_player: AnimationPlayer = $AnimationPlayer
@onready var action_timer: Timer = $ActionTimer
@onready var lifespan_timer: Timer = $LifespanTimer 

func _ready():
	action_timer.timeout.connect(_on_action_timer_timeout)
	lifespan_timer.timeout.connect(_on_lifespan_timer_timeout) 

	if animation_player.has_animation("spawn"):
		animation_player.play("spawn")
	else:
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

	if label: # Update label after setting type
		update_label()

	# Special setup for certain types
	if card_type == CardDefs.CardType.FLEETING_MOMENT:
		if lifespan_timer and card_properties.has("lifespan"):
			lifespan_timer.wait_time = card_properties.lifespan
			lifespan_timer.start()

func update_label():
	if not label: return 
	var base_label = CardDefs.get_label(card_type)
	var extra_info = ""

	match card_type:
		CardDefs.CardType.GARDENER:
			if card_properties.has("energy"):
				extra_info = "\nEnergy: %d%%" % int(card_properties.energy / card_properties.max_energy * 100.0)
		CardDefs.CardType.CHRONO_CRYSTAL:
			if card_properties.has("energy"):
				extra_info = "\nCharge: %d" % int(card_properties.energy)
		CardDefs.CardType.FLEETING_MOMENT:
			if lifespan_timer and not lifespan_timer.is_stopped():
				extra_info = "\n%.1fs left" % lifespan_timer.time_left

	label.text = base_label + extra_info

func start_action_timer(duration: float):
	if not is_working:
		is_working = true
		action_timer.wait_time = duration
		action_timer.start()
		play_feedback_animation("working")
		sprite.modulate = Color(0.8, 0.8, 0.8) # Dim slightly

func _on_action_timer_timeout():
	is_working = false
	emit_signal("action_complete", self)
	play_feedback_animation("complete")
	sprite.modulate = Color.WHITE

func _on_lifespan_timer_timeout():
	print(CardDefs.get_label(card_type), " faded away.")
	self_destruct.emit(self)
	consume(true)

func get_overlapping_cards() -> Array[Card]:
	var overlapping_cards : Array[Card] = []
	var areas = get_overlapping_areas()
	for area in areas:
		if area is Card and area != self:
			overlapping_cards.append(area)
	return overlapping_cards

func consume(play_anim: bool = true):
	if play_anim:
		is_working = true # Prevent interaction while fading
		play_feedback_animation("consumed")
		await get_tree().create_timer(0.2).timeout
	queue_free()

func is_growable_plant() -> bool:
	return card_type in [CardDefs.CardType.SUNPETAL_SEED, CardDefs.CardType.SUNPETAL_SPROUT]

func is_temporal_energy() -> bool:
	return card_type in [CardDefs.CardType.PAST_DEW, CardDefs.CardType.FLEETING_MOMENT]

func is_harvestable_plant() -> bool:
	return card_type == CardDefs.CardType.SUNPETAL_MATURE

func play_feedback_animation(anim_name: String):
	if not animation_player: return
	match anim_name:
		"pickup": 
			animation_player.play("pickup")
		"drop":
			animation_player.play("drop")
		"working":
			animation_player.play("working")
		"complete":
			animation_player.stop(); position.y = 0
			animation_player.play("complete")
		"consumed":
			animation_player.play("consumed")

# Helper to quickly create simple animations if they don't exist
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
	animation_player.add_animation(name, anim)
