extends Node
class_name WorldController

@export var spawn_point:Node3D

func _ready() -> void:
	GlobalWorldAccess.current_world = self

func _physics_process(_delta: float) -> void:
	if (NetworkManager as NetNodeManager).has_message() and (NetworkManager as NetNodeManager).get_message_type() == 6:
		var player:MovementHandler = preload("res://scenes/player/player.tscn").instantiate()
		player.position = spawn_point.global_position
		player.rotation = spawn_point.global_rotation
		player.networker.owner_id = (NetworkManager as NetNodeManager).get_message_player()
		(NetworkManager as NetNodeManager).pop_message()
		add_child.call_deferred(player)
	
	while !(NetworkManager as NetNodeManager).id_ready():
		await get_tree().physics_frame
	if (NetworkManager as NetNodeManager).get_id() == 0:
		if (NetworkManager as NetNodeManager).has_message() and (NetworkManager as NetNodeManager).get_message_type() == 3:
			print(str((NetworkManager as NetNodeManager).get_message_player()) + ": " + (NetworkManager as NetNodeManager).peek_message()[0])
			(NetworkManager as NetNodeManager).pop_message()
