extends Node

signal avatar_loaded

@export var networker:PlayerNetworker
@export var player:Player
var owner_id:int
var equiped_avatar:Node3D
var current_avatar:int
var new_avatar:PackedScene

func _ready() -> void:
	@warning_ignore("unsafe_property_access")
	owner_id = networker.owner_id
	GlobalWorldAccess.current_world.avatar_change_handler.avatar_changed.connect(change_avatar)
	while !(NetworkManager as NetNodeManager).id_ready():
		await get_tree().physics_frame
	if owner_id == (NetworkManager as NetNodeManager).get_id():
		GlobalWorldAccess.current_world.avatar_change_handler.send_message(owner_id, 0)

func change_avatar(target_player:int, avatar:int) -> void:
	if target_player != owner_id:
		return
	if equiped_avatar != null:
		equiped_avatar.queue_free()
	if avatar == 0:
		new_avatar = preload("res://scenes/player/avatars/built in avatars/humanoid/humanoid.tscn")
	else:
		new_avatar = preload("res://scenes/player/avatars/built in avatars/loading_avatar.tscn")
		current_avatar = avatar
		equiped_avatar = new_avatar.instantiate()
		get_parent().add_child(equiped_avatar)
		AvatarPackLoader.update_avatar_list()
		var thread:Thread = Thread.new()
		thread.start(load_avatar_on_thread.bind(avatar))
		await avatar_loaded
		thread.wait_to_finish()
	current_avatar = avatar
	if equiped_avatar:
		equiped_avatar.queue_free()
		await get_tree().physics_frame # dont have both avatars loaded at the same time
	equiped_avatar = new_avatar.instantiate()
	setup_avatar(equiped_avatar)
	get_parent().add_child(equiped_avatar)
	player.avatar_changed.emit()

func load_avatar_on_thread(avatar:int) -> void:
	if !AvatarPackLoader.avatars.has(avatar):
		new_avatar = preload("res://scenes/player/avatars/built in avatars/missing_avatar.tscn")
		avatar_loaded.emit()
		return
	new_avatar = load(AvatarPackLoader.avatars[avatar].scene_path)
	# abort if source avatar is unsafe unless unsafe loading is enabled
	if (!check_safe(new_avatar.get_state())):
		if !OS.get_cmdline_args().has("--unsafe-load"):
			push_warning("***DANGER*** tried to load unsafe avatar! aborting. to override this run with \"--unsafe-load\"")
			new_avatar = preload("res://scenes/player/avatars/built in avatars/missing_avatar.tscn")
		else:
			push_warning("safety check disabled: loading unsafe avatar, this is very dangerous!")
	avatar_loaded.emit.call_deferred()

# checks if the scene is capable of code execution, probably not foolproof
func check_safe(root:SceneState) -> bool:
	# check for scripts
	for idx:int in range(root.get_node_count()):
		for property_idx:int in range(root.get_node_property_count(idx)):
			if root.get_node_property_name(idx, property_idx) == "script":
				if root.get_node_property_value(idx, property_idx) != null:
					return false
	return true

func get_node_and_children_recursive(root:Node) -> Array[Node]:
	var nodes:Array[Node]
	nodes.append(root)
	for node:Node in root.get_children():
		nodes.append_array(get_node_and_children_recursive(node))
	return nodes

# goes through the avatar scene looking for stubs and replaces them with the corrosponding scenes
func setup_avatar(root:Node) -> void:
	var combined_aabb:AABB = AABB(Vector3(0, 0, 0), Vector3(0.1, 0.1, 0.1))
	var nodes:Array[Node] = get_node_and_children_recursive(root)
	for node:Node in nodes:
		if node is VisualInstance3D:
			var aabb:AABB = (node as VisualInstance3D).get_aabb().abs()
			combined_aabb.merge(aabb)
		if node.has_meta("IKMarker") and node is Skeleton3D:
			var ik:AvatarIK = preload("res://scenes/player/avatars/godot_ik.tscn").instantiate()
			var values:Dictionary = node.get_meta("IKMarker")
			# todo: check marker is valid
			node.add_child(ik)
			ik.head_bone = values["head_bone"]
			ik.left_arm_bone = values["left_arm_bone"]
			ik.right_arm_bone = values["right_arm_bone"]
			ik.left_leg_bone = values["left_leg_bone"]
			ik.right_leg_bone = values["right_leg_bone"]
			ik.spine_bone = values["spine_bone"]
			ik.hip_bone = values["hip_bone"]
			ik.head_target = node.get_child(values["head_target"])
			ik.left_arm_target = node.get_child(values["left_arm_target"])
			ik.right_arm_target = node.get_child(values["right_arm_target"])
			ik.spine_target = node.get_child(values["spine_target"])
			ik.hip_target = node.get_child(values["hip_target"])
			ik.setup(player.is_local)
			player.head_ik_target = ik.head.target
			player.left_arm_ik_target = ik.left_arm.target
			player.right_arm_ik_target = ik.right_arm.target
			@warning_ignore("unsafe_call_argument", "unsafe_property_access")
			player.head_view_offset = node.get_child(values["head_view"]).position - node.get_child(values["head_target"]).position
	(player.collider.shape as CapsuleShape3D).radius = maxf(combined_aabb.size.x, combined_aabb.size.z)
	(player.collider.shape as CapsuleShape3D).height = combined_aabb.size.y
	player.collider.position = combined_aabb.position
