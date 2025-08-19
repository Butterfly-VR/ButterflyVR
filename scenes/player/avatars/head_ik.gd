extends GodotIKEffector

var target:Node3D
var is_local:bool

func _ready() -> void:
	if is_local:
		scale = Vector3(0.01, 0.01, 0.01)

func _physics_process(_delta: float) -> void:
	if target != null:
		rotation = target.rotation
		global_position = target.global_position
