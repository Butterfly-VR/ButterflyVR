extends GodotIKEffector

var target:Node3D

func _physics_process(_delta: float) -> void:
	if target == null:
		target = get_parent().get_parent().get_parent().get_parent().get_parent().get_parent().get_node_or_null("Camera3D")
		if target != null:
			scale = Vector3(0.001, 0.001, 0.001)
		else:
			target = get_parent().get_parent().get_parent().get_parent().get_parent().get_parent().get_node_or_null("PivotCenter")

	if target != null:
		rotation = -target.rotation # think something gets flipped somewhere in this scene but we end up needing to invert rotation here
		global_position = target.global_position
