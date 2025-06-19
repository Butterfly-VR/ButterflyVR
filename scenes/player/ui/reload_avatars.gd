extends Button

func _pressed() -> void:
	for node:Node in get_tree().get_nodes_in_group("RemoteAvatarManagers"):
		node.change_avatar(node.current_avatar)
