extends CanvasLayer

@onready var biodiversity_label: Label = %BiodiversityLabel 
@onready var gardener_focus_label: Label = %GardenerEnergyLabel
@onready var status_label: Label = %StatusLabel

func update_biodiversity(score: int):
	biodiversity_label.text = "Biodiversity: %d" % score

func update_gardener_focus(focus: float, max_focus: float):
	if max_focus > 0:
		gardener_focus_label.text = "Gardener Focus: %d%%" % int(focus / max_focus * 100.0)
	else:
		gardener_focus_label.text = "Gardener Focus: N/A"

func show_win_message():
	show_status("GENESIS BLOOM CULTIVATED! The Ecosystem Thrives!")
	status_label.modulate = Color.PALE_GREEN

func show_lose_message(reason: String):
	show_status("ECOSYSTEM COLLAPSED: %s" % reason)
	status_label.modulate = Color.DARK_RED

func show_status(message: String):
	status_label.text = message
	status_label.modulate = Color.WHITE

func clear_status():
	status_label.text = ""
