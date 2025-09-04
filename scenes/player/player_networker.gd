extends NetworkedNode
class_name PlayerNetworker

@export var target:Player

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
	var values:Array = []
	values.push_back(target.position)
	values.push_back(target.rotation)
	values.push_back(target.velocity)
	values.push_back(target.head_ik_target.position)
	values.push_back(target.head_ik_target.rotation)
	values.push_back(target.left_arm_ik_target.position)
	values.push_back(target.left_arm_ik_target.rotation)
	values.push_back(target.right_arm_ik_target.position)
	values.push_back(target.right_arm_ik_target.rotation)
	values.push_back(target.interactor_origin.position)
	values.push_back(target.interactor_origin.rotation)
	return values

func _set_networked_values(values: Array) -> void:
	target.position = values[0]
	target.rotation = values[1]
	target.velocity = values[2]
	target.head_ik_target.position = values[3]
	target.head_ik_target.rotation = values[4]
	target.left_arm_ik_target.position = values[5]
	target.left_arm_ik_target.rotation = values[6]
	target.right_arm_ik_target.position = values[7]
	target.right_arm_ik_target.rotation = values[8]
	target.interactor_origin.position = values[9]
	target.interactor_origin.rotation = values[10]

func _get_networked_value_type(idx: int) -> int:
	match idx:
		0:
			return 5
		1:
			return 5
		2:
			return 5
		3:
			return 5
		4:
			return 5
		5:
			return 5
		6:
			return 5
		7:
			return 5
		8:
			return 5
		9:
			return 5
		10:
			return 5
	return -1

func _get_priority(_clientid: int) -> int:
	return 1000

func _on_owner_dc() -> void:
	target.queue_free()
