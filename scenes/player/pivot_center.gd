extends Node3D

@export var player_access:PlayerAccess
var player:Player

func _ready() -> void:
	player = player_access.player
	player.interactor_origin = self
