extends NetworkedNode
class_name PlayerNetworker

@export var target:MovementHandler

func _ready() -> void:
	while !(NetworkManager as NetNodeManager).id_ready():
		await get_tree().physics_frame
	if owner_id == (NetworkManager as NetNodeManager).get_id():
		target.init_local()
	else:
		target.init_remote()
	if (NetworkManager as NetNodeManager).is_server():
		(NetworkManager as NetNodeManager).register_player_object(owner_id, target)

func _get_networked_values() -> Array:
	return [target.position, target.rotation, target.velocity, target.cam_x_rotation, target.player_state == target.Player_states.CROUCHED]

func _set_networked_values(values: Array) -> void:
	target.position = values[0]
	target.rotation = values[1]
	target.velocity = values[2]
	target.cam_x_rotation = values[3]
	if values[4]:
		target.player_state = target.Player_states.CROUCHED
	else:
		target.player_state = target.Player_states.NONE

func _get_networked_value_type(idx: int) -> int:
	match idx:
		0:
			return 5
		1:
			return 5
		2:
			return 5
		3:
			return 4
		4:
			return 0
	return -1

func _get_priority(_clientid: int) -> int:
	return 1000

func _on_owner_dc() -> void:
	target.queue_free()
