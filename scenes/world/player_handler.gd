extends MessageHandler

func on_player_join(player:int) -> void:
	send_message_final([player], [_get_value_type(null, 0)])

func _ready() -> void:
	(NetworkManager as NetNodeManager).player_joined.connect(on_player_join)

func _get_value_type(_previous_value: Variant, idx: int) -> int:
	match idx:
		0:
			return 2
	return -1

func _process_message(values: Array) -> void:
	var player_owner:int = values[0]
	var world:WorldController = GlobalWorldAccess.current_world
	var player:Player = preload("res://scenes/player/player.tscn").instantiate()
	player.set_meta("owner_id", player_owner)
	world.add_child(player)
	player.global_transform = world.spawn_point.global_transform
