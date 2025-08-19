extends GodotIK

func setup(head_bone:int, left_arm_bone:int, right_arm_bone:int, left_leg_bone:int, right_leg_bone:int, is_local:bool, head_target:Node3D, player:MovementHandler) -> void:
	head_target.global_transform = (get_child(4) as GodotIKEffector).global_transform
	player.camera_height = head_target.position.y
	(get_child(4) as GodotIKEffector).bone_idx = head_bone
	(get_child(4) as GodotIKEffector).is_local = is_local
	(get_child(4) as GodotIKEffector).target = head_target
	(get_child(1) as GodotIKEffector).bone_idx = left_arm_bone
	(get_child(0) as GodotIKEffector).bone_idx = right_arm_bone
	(get_child(3) as GodotIKEffector).bone_idx = left_leg_bone
	(get_child(2) as GodotIKEffector).bone_idx = right_leg_bone
