extends Node

func _physics_process(_delta: float) -> void:
	if (NetworkManager as NetNodeManager).has_message() and (NetworkManager as NetNodeManager).get_message_type() == 4:
		on_dc((NetworkManager as NetNodeManager).get_message_player())


func on_dc(player:int) -> void:
	await get_tree().physics_frame
	for node:NetworkedNode in (NetworkManager as NetNodeManager).get_networked_nodes():
		if node.owner_id == player:
			node._on_owner_dc()
	(NetworkManager as NetNodeManager).pop_message()
