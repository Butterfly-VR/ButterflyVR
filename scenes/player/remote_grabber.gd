extends Node3D

@export var player_access:PlayerAccess
var grabbed_node:Node3D
var owner_id:int
var networker:PlayerNetworker
var player:Player

func _ready() -> void:
	player = player_access.player
	networker = player_access.networker
	owner_id = networker.owner_id
	GlobalWorldAccess.current_world.player_grab_handler.player_grabbed.connect(on_grab)

func _physics_process(_delta: float) -> void:
	if grabbed_node != null:
		grabbed_node.global_position = global_position

func on_release() -> void:
	if grabbed_node is RigidBody3D:
		(grabbed_node as RigidBody3D).freeze = false
	grabbed_node = null

func on_grab(grabbing_player:int, target:Node) -> void:
	if grabbing_player != owner_id:
		return
	if target == null:
		on_release()
		return
	if target is Node3D:
		global_position = (target as Node3D).global_position
		grabbed_node = target
		if grabbed_node is RigidBody3D:
			(grabbed_node as RigidBody3D).freeze = true
