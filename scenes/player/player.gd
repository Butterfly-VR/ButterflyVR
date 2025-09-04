extends CharacterBody3D
class_name Player

@export var networker:PlayerNetworker
@export var collider:CollisionShape3D

signal avatar_changed

var head_ik_target:Node3D = Node3D.new()
var left_arm_ik_target:Node3D = Node3D.new()
var right_arm_ik_target:Node3D = Node3D.new()
var interactor_origin:Node3D = Node3D.new()
var head_view_offset:Vector3
var player_logic:PlayerAccess
var is_local:bool

func init_local() -> void:
	is_local = true
	player_logic = preload("res://scenes/player/local_player_desktop.tscn").instantiate()
	final_init()

func init_remote() -> void:
	is_local = false
	player_logic = preload("res://scenes/player/remote_player.tscn").instantiate()
	final_init()

func final_init() -> void:
	player_logic.player = self
	player_logic.networker = networker
	add_child.call_deferred(player_logic)

func _physics_process(_delta:float) -> void:
	move_and_slide()
