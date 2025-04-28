class_name UI extends CanvasLayer

@onready var crystal_energy_label: Label = %CrystalEnergyLabel
@onready var gardener_energy_label: Label = %GardenerEnergyLabel
@onready var status_label: Label = %StatusLabel

func update_crystal_energy(energy: float):
	crystal_energy_label.text = "Crystal Energy: %d" % int(energy)

func update_gardener_energy(energy: float, max_energy: float):
	if max_energy > 0:
		gardener_energy_label.text = "Gardener Energy: %d%%" % int(energy / max_energy * 100.0)
	else:
		gardener_energy_label.text = "Gardener Energy: N/A"

func show_status(message: String):
	status_label.text = message
	status_label.modulate = Color.WHITE

func show_win_message():
	show_status("AEVUM BLOOM CULTIVATED! The Garden is Saved!")
	status_label.modulate = Color.LIGHT_GREEN

func show_lose_message(reason: String):
	show_status("GAME OVER: %s" % reason)
	status_label.modulate = Color.ORANGE_RED

func clear_status():
	status_label.text = ""
