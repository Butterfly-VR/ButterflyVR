extends MeshInstance3D
class_name Highlighter

var mat:StandardMaterial3D = StandardMaterial3D.new()

var geometry:MeshInstance3D

func _ready() -> void:
	visible = false
	if get_parent() is not MeshInstance3D:
		push_warning("mesh outliner was not child of mesh")
		queue_free()
		return
	geometry = get_parent()
	mesh = geometry.mesh
	mat.albedo_color = Color(0.0, 0.0, 1.0)
	mat.cull_mode = BaseMaterial3D.CULL_FRONT
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	scale = geometry.scale * 1.05
	set_surface_override_material(0, mat)
