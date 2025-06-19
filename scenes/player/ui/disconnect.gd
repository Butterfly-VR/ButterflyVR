extends Button

func _pressed() -> void:
	(NetworkManager as NetNodeManager).stop()
	get_tree().change_scene_to_packed(preload("res://scenes/startup/loading.tscn"))
