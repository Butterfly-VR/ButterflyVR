extends Camera3D
class_name DesktopCamera

const VERTICAL_LIMIT_MAX:float = 1.5707
const VERTICAL_LIMIT_MIN:float = -1.5707

@export var player_access:PlayerAccess
@export var raycaster:Node3D
@export var movement_handler:MovementHandler
@export var rotation_center:Node3D

var sensitivity:float = 0.0005
var verticalSensitivityMultiplier:float = 0.75
var head_target_ready:bool = false

@onready var horizontalSensitivity:float = sensitivity
@onready var verticalSensitivity:float = sensitivity * verticalSensitivityMultiplier

func _ready() -> void:
	Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
	player_access.player.interactor_origin = raycaster
	player_access.player.avatar_changed.connect(on_avatar_changed)

func on_avatar_changed() -> void:
	rotation_center.global_position = player_access.player.head_ik_target.global_position
	position = player_access.player.head_view_offset
	if movement_handler.player_state == MovementHandler.Player_states.CROUCHED:
		movement_handler.player_state = MovementHandler.Player_states.NONE
	movement_handler.camera_height = rotation_center.position.y
	head_target_ready = true

func _unhandled_input(event:InputEvent) -> void:
	if event is InputEventMouseMotion and Input.get_mouse_mode() == Input.MOUSE_MODE_CAPTURED:
		move_cam((event as InputEventMouseMotion).relative.x, (event as InputEventMouseMotion).relative.y)

func move_cam(xrot:float, yrot:float) -> void:
	rotation_center.rotate_y(-xrot * horizontalSensitivity)
	rotation_center.rotation.x = clamp(rotation_center.rotation.x - (yrot * verticalSensitivity), VERTICAL_LIMIT_MIN, VERTICAL_LIMIT_MAX)

func _physics_process(_delta: float) -> void:
	if head_target_ready:
		player_access.player.head_ik_target.global_position = rotation_center.global_position
		player_access.player.head_ik_target.global_basis = rotation_center.global_basis
