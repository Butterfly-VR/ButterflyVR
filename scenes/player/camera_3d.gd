extends Camera3D
class_name DesktopCamera

# dont rotate up or down more than 90 degrees
const VERTICAL_LIMIT_MAX:float = 1.5707
const VERTICAL_LIMIT_MIN:float = -1.5707

@export var player_access:PlayerAccess

var rotateTarget:MovementHandler # the node the camera should rotate when turning left or right
# future proofing for when these become changable settings
var sensitivity:float = 0.0005
var verticalSensitivityMultiplier:float = 0.75
@onready var horizontalSensitivity:float = sensitivity
@onready var verticalSensitivity:float = sensitivity * verticalSensitivityMultiplier

func _ready() -> void:
	rotateTarget = player_access.player
	Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
	player_access.player.head = self

func _unhandled_input(event:InputEvent) -> void:
	if event is InputEventMouseMotion and Input.get_mouse_mode() == Input.MOUSE_MODE_CAPTURED:
		move_cam((event as InputEventMouseMotion).relative.x, (event as InputEventMouseMotion).relative.y)

func move_cam(xrot:float, yrot:float) -> void:
	rotateTarget.rotation.y -= xrot * horizontalSensitivity
	rotation.x = clamp(rotation.x - (yrot * verticalSensitivity), VERTICAL_LIMIT_MIN, VERTICAL_LIMIT_MAX)
	rotateTarget.cam_x_rotation = rotation.x
