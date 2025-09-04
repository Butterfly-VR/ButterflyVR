extends GodotIKEffector
class_name IKFollower

var target:Node3D
var follow_target:bool = true
var is_ready:bool = false
var original_scale:Vector3
@export var secondary_target1:Node3D
@export_range(0, 1) var secondary_target1_strength:float
@export var secondary_target2:Node3D

func _ready() -> void:
	original_scale = scale
	global_position = target.global_position
	global_basis = target.global_basis
	await get_tree().physics_frame
	is_ready = true
	if secondary_target1:
		secondary_target1.global_position = global_position
		secondary_target1.global_basis = global_basis
	if secondary_target2:
		secondary_target2.global_position = global_position
		secondary_target2.global_basis = global_basis

func _physics_process(_delta: float) -> void:
	if !is_ready:
		return
	if follow_target:
		global_basis = target.global_basis
		global_position = target.global_position
	else:
		if secondary_target2:
			global_position = (secondary_target1.global_position * secondary_target1_strength) + (secondary_target2.global_position * (1 - secondary_target1_strength))
			global_basis = secondary_target2.global_basis.orthonormalized().slerp(secondary_target1.global_basis.orthonormalized(), secondary_target1_strength)
		else:
			global_position = secondary_target1.global_position
			global_basis = secondary_target1.global_basis
		target.global_basis = global_basis
		target.global_position = global_position
	scale = original_scale
