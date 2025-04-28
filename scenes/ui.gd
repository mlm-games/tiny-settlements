extends CanvasLayer

@onready var day_label: Label = %DayLabel 
@onready var time_label: Label = %TimeLabel
@onready var hunger_label: Label = %HungerLabel
@onready var status_label: Label = %StatusLabel

func update_day(day: int):
	day_label.text = "Day: %d" % day

func update_time(time_left: float):
	time_label.text = "Time: %d" % int(time_left)

func update_hunger(hunger: float, max_hunger: float):
	if max_hunger > 0:
		hunger_label.text = "Villager Hunger: %d%%" % int(hunger / max_hunger * 100.0)
	else:
		hunger_label.text = "Villager Hunger: N/A"

func show_status(message: String):
	status_label.text = message
	status_label.modulate = Color.WHITE # Fixes it appearing invisible sometimes

func show_win_message():
	show_status("YOU BUILT THE HUT! YOU WIN!")
	status_label.modulate = Color.GOLD

func show_lose_message(reason: String):
	show_status("GAME OVER: %s" % reason)
	status_label.modulate = Color.CRIMSON

func clear_status():
	status_label.text = ""
