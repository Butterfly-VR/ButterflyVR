extends Node
# will later contain login for stuff like auto login, showing the login screen, loading home world, etc

@export var token_box:TextEdit

func _ready() -> void:
	AvatarPackLoader.update_avatar_list()
	if DisplayServer.get_name() == "headless":
		get_tree().change_scene_to_file.call_deferred("res://scenes/startup/loading_server.tscn")
		return
	for argument:String in OS.get_cmdline_args():
		if argument == "--server":
			get_tree().change_scene_to_file.call_deferred("res://scenes/startup/loading_server.tscn")
			return
	# client path
	(get_child(0) as CanvasItem).visible = true
	@warning_ignore("unsafe_property_access", "unsafe_cast")
	(token_box.token_entered as Signal).connect(start_client)

func start_client(token:PackedByteArray) -> void:
	(NetworkManager as NetNodeManager).start_client(token)
	get_tree().change_scene_to_file.call_deferred("res://scenes/world/debug_world.tscn")
