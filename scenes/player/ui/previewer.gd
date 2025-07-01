extends Node3D

@export var list:VBoxContainer
var avatar:int = -1
var new_avatar:PackedScene

signal avatar_preview_loaded

func _physics_process(_delta: float) -> void:
	@warning_ignore("unsafe_property_access")
	if avatar != list.selected_avatar:
		for child:Node in get_children():
			child.free()
		@warning_ignore("unsafe_property_access")
		avatar = list.selected_avatar
		if avatar == 0:
			new_avatar = preload("res://scenes/player/avatars/built in avatars/cubefella/cube_fella.tscn")
		else:
			AvatarPackLoader.update_avatar_list()
			var thread:Thread = Thread.new()
			thread.start(load_avatar_on_thread.bind(avatar))
			await avatar_preview_loaded
			thread.wait_to_finish()
		add_child(setup_avatar(new_avatar.instantiate()))

func load_avatar_on_thread(avatar:int) -> void:
	if !AvatarPackLoader.avatars.has(avatar):
		new_avatar = preload("res://scenes/player/avatars/built in avatars/missing_avatar.tscn")
		avatar_preview_loaded.emit()
		return
	new_avatar = load(AvatarPackLoader.avatars[avatar].scene_path)
	# abort if source avatar is unsafe unless unsafe loading is enabled
	if (!check_safe(new_avatar.get_state())):
		if !OS.get_cmdline_args().has("--unsafe-load"):
			push_warning("***DANGER*** tried to load unsafe avatar! aborting. to override this run with \"--unsafe-load\"")
			new_avatar = preload("res://scenes/player/avatars/built in avatars/missing_avatar.tscn")
		else:
			push_warning("safety check disabled: loading unsafe avatar, this is very dangerous!")
	avatar_preview_loaded.emit.call_deferred()

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
	for node:Node in root.get_children():
		nodes.append_array(get_node_and_children_recursive(node))
		nodes.append(node)
	return nodes

# goes through the avatar scene looking for stubs and replaces them with the corrosponding scenes
func setup_avatar(root:Node) -> Node:
	var nodes:Array[Node] = get_node_and_children_recursive(root)
	# check for markers and setup if they exist
	for node:Node in nodes:
		if node.has_meta("IKMarker"):
			var ik:GodotIK = preload("res://scenes/player/avatars/godot_ik.tscn").instantiate()
			var bones:Dictionary = node.get_meta("IKMarker")
			# todo: check marker is valid
			@warning_ignore("unsafe_method_access")
			ik.setup(bones["head_bone"], bones["left_arm_bone"], bones["right_arm_bone"], bones["left_leg_bone"], bones["right_leg_bone"])
			node.add_child(ik)
	return root
