extends RayCast3D

var current_target:Node

func _physics_process(_delta: float) -> void:
	if is_colliding():
		if get_collider().has_method("start_highlight"):
			current_target = get_collider()
			current_target.start_highlight()
	elif current_target:
		if current_target.has_method("end_highlight"):
			current_target.end_highlight()
			current_target = null
