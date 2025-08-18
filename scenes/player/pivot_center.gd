extends Node3D

@export var player_access:PlayerAccess
var player:MovementHandler

func _ready() -> void:
	player = player_access.player
	player.head = self

func _physics_process(_delta: float) -> void:
	if player.cam_x_rotation != null:
		rotate_x(player.cam_x_rotation - rotation.x)
