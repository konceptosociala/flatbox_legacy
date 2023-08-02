local Transform = TransformWrapper.get;

print("Transform(x: "..Transform.translation.x..", y: "..Transform.translation.y..", z: "..Transform.translation.z..")")

Transform.translation = {
    x = Transform.translation.x + 0.01,
    y = Transform.translation.y + 0.01,
    z = Transform.translation.z + 0.01,
}

TransformWrapper:set(Transform)