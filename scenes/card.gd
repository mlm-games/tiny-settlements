extends Area2D
class_name Card

const CardDefs = preload("uid://drx32as7xbxxr")

signal drag_started(card)
signal drag_ended(card, dropped_on_cards)
signal action_complete(card) # When ActionTimer finishes

@export var card_type : CardDefs.CardType = CardDefs.CardType.NONE : set = set_card_type
var card_properties : Dictionary = {} # Specific data like hunger, durability

var is_dragging: bool = false
var is_working: bool = false # Is this card busy with a timed action?
var drag_offset: Vector2 = Vector2.ZERO

@onready var sprite: Sprite2D = $Sprite2D
@onready var label: Label = %Label
@onready var collision_shape: CollisionShape2D = $CollisionShape2D
@onready var animation_player: AnimationPlayer = $AnimationPlayer
@onready var action_timer: Timer = $ActionTimer

func _ready():
	action_timer.timeout.connect(_on_action_timer_timeout)

	if animation_player.has_animation("spawn"):
		animation_player.play("spawn")
	else: # ik lazy
		scale = Vector2.ZERO
		var tween = create_tween()
		tween.tween_property(self, "scale", Vector2.ONE, 0.2).set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT).from(Vector2.ZERO)

	set_card_type(card_type) 

func _input_event(viewport, event, shape_idx):
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT:
			if event.pressed and not is_working: # Can't drag if busy
				is_dragging = true
				drag_offset = get_global_mouse_position() - global_position
				z_index = 10 # To bring to front visually
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
	if label: label.text = CardDefs.get_label(card_type)

	if CardDefs.CARD_PROPERTIES.has(card_type):
		card_properties = CardDefs.CARD_PROPERTIES[card_type].duplicate() # Copy properties
		if label: update_label()

func update_label():
	var base_label = CardDefs.get_label(card_type)
	var extra_info = ""

	match card_type:
		CardDefs.CardType.VILLAGER:
			if card_properties.has("hunger"):
				extra_info = "\nHunger: %d%%" % int(card_properties.hunger)
		CardDefs.CardType.TREE, CardDefs.CardType.ROCK_PILE, CardDefs.CardType.BERRY_BUSH:
			if card_properties.has("durability"):
				extra_info = "\nUses: %d" % card_properties.durability

	label.text = base_label + extra_info

func start_action_timer(duration: float):
	if not is_working:
		is_working = true
		action_timer.wait_time = duration
		action_timer.start()
		play_feedback_animation("working")
		sprite.modulate = Color(0.8, 0.8, 0.8)

func _on_action_timer_timeout():
	is_working = false
	emit_signal("action_complete", self)
	play_feedback_animation("complete")
	sprite.modulate = Color.WHITE

func get_overlapping_cards() -> Array[Card]:
	var overlapping_cards : Array[Card] = []
	var areas = get_overlapping_areas()
	for area in areas:
		if area is Card and area != self: # Ensure it's the other card
			overlapping_cards.append(area)
	return overlapping_cards

func consume(play_anim: bool = true):
	if play_anim:
		play_feedback_animation("consumed")
		await get_tree().create_timer(0.2).timeout
	queue_free()

func decrease_durability():
	if card_properties.has("durability"):
		card_properties.durability -= 1
		update_label()
		if card_properties.durability <= 0:
			consume() 
			return true
	return false # Return false if not depleted

func play_feedback_animation(anim_name: String):
	#if not animation_player: return

	# Define simple animations here or create them in the editor
	match anim_name:
		"pickup":
			animation_player.play("pickup")
		"drop":
			animation_player.play("drop")
		"working":
			animation_player.play("working")
		"complete":
			animation_player.stop() # Stop (working animation)
			animation_player.play("RESET") # Reset position if working animation moved it
			animation_player.play("complete")
		"consumed":
			animation_player.play("consumed")
		"spawn": 
			pass
