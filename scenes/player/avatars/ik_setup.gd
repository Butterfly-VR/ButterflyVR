extends GodotIK
class_name AvatarIK

@export var head:IKFollower
@export var left_arm:IKFollower
@export var right_arm:IKFollower
@export var left_leg:GodotIKEffector
@export var right_leg:GodotIKEffector
@export var spine:IKFollower
@export var hips:IKFollower

var head_bone:int
var left_arm_bone:int
var right_arm_bone:int
var left_leg_bone:int
var right_leg_bone:int
var spine_bone:int
var hip_bone:int
var head_target:Node3D
var left_arm_target:Node3D
var right_arm_target:Node3D
var spine_target:Node3D
var hip_target:Node3D
var follow_target_spine:bool
var follow_target_hip:bool
var follow_target_left_arm:bool
var follow_target_right_arm:bool
var follow_target_left_leg:bool
var follow_target_right_leg:bool

func setup(is_local:bool) -> void:
	head.bone_idx = head_bone
	head.target = head_target
	head.scale_local()
	left_arm.bone_idx = left_arm_bone
	left_arm.target = left_arm_target
	left_arm.follow_target = follow_target_left_arm
	right_arm.bone_idx = right_arm_bone
	right_arm.target = right_arm_target
	right_arm.follow_target = follow_target_right_arm
	left_leg.bone_idx = left_leg_bone
	#left_leg.follow_target = follow_target_left_leg # todo: have leg ik controllers derive from IKFollower
	right_leg.bone_idx = right_leg_bone
	#right_leg.follow_target = follow_target_right_leg
	spine.bone_idx = spine_bone
	spine.target = spine_target
	spine.follow_target = follow_target_spine
	hips.bone_idx = hip_bone
	hips.target = hip_target
	hips.follow_target = follow_target_hip
	
