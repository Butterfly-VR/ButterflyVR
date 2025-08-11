extends Button

func _pressed() -> void:
	var tree:SceneTree = get_tree()
	get_parent().remove_child(self)
	GlobalWorldAccess.current_world.queue_free()
	await tree.physics_frame
	(NetworkManager as NetNodeManager).stop()
	tree.change_scene_to_packed(preload("res://scenes/startup/loading.tscn"))
