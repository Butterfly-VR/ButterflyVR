extends NetworkedNode
class_name RigidBodyNetworker

@export var target:NetworkedRigidBody

func _get_networked_values() -> Array:
	return [target.position, target.rotation, target.linear_velocity, target.sleeping, target.freeze]

func _set_networked_values(values: Array) -> void:
	target.has_update = true
	target.pos = values[0]
	target.rot = values[1]
	target.vel = values[2]
	target.sle = values[3]
	target.fre = values[4]


func _get_networked_value_type(idx: int) -> int:
	match idx:
		0:
			return 5
		1:
			return 5
		2:
			return 5
		3:
			return 0
		4:
			return 0
	return -1

func _get_priority(_clientid: int) -> int:
	return 10
