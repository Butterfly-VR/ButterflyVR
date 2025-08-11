extends MessageHandler

func on_player_left(player:int) -> void:
	send_message_final([player], [_get_value_type(null, 0)])

func _ready() -> void:
	(NetworkManager as NetNodeManager).player_left.connect(on_player_left)

func _get_value_type(_previous_value: Variant, idx: int) -> int:
	match idx:
		0:
			return 2
	return -1
func _process_message(values: Array) -> void:
	handle_on_dc(values)

func handle_on_dc(values: Array) -> void:
	var player:int = values[0]
	for node:NetworkedNode in (NetworkManager as NetNodeManager).get_networked_nodes():
		if node.owner_id == player:
			node._on_owner_dc()
