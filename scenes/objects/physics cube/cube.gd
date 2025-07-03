extends RigidBody3D
class_name NetworkedRigidBody

var pos:Vector3
var rot:Vector3
var vel:Vector3
var ang:Vector3
var sle:bool
var fre:bool
var has_update:bool
var has_update_integrate:bool
@export var networker:RigidBodyNetworker
@export var mesh:MeshInstance3D
@export var interactor:Interactable
func _ready() -> void:
	interactor.interacted_primary.connect(on_interact)
func on_interact() -> void:
	(mesh.get_active_material(0) as StandardMaterial3D).albedo_color = Color8(randi_range(0, 255), randi_range(0, 255), randi_range(0, 255))
func _physics_process(_delta: float) -> void:
	if has_update:
		has_update = false
		position = pos
		rotation = rot
		sleeping = sle
		freeze = fre
func _integrate_forces(state: PhysicsDirectBodyState3D) -> void:
	if has_update_integrate:
		has_update_integrate = false
		state.linear_velocity = vel
		state.angular_velocity = ang
func on_grab() -> void:
	(NetworkManager as NetNodeManager).become_object_owner(networker.objectid)
func on_release() -> void:
	(NetworkManager as NetNodeManager).release_object_owner(networker.objectid)

func start_highlight() -> void:
	@warning_ignore("unsafe_property_access")
	mesh.get_child(0).visible = true

func end_highlight() -> void:
	@warning_ignore("unsafe_property_access")
	mesh.get_child(0).visible = false
