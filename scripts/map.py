# Before edit map data, :set lsp=4 without mode lines.
def test_map():
    """Return the map for tests."""
    floors = []
    # map position
    position = [0.0, 0.0, 0.0]
    # 1st floor
    height = 0
    data = [
        [ 0, 1, 2, 3,],
        [ 0,-1,-1, 0,],
        [ 0, 0, 0, 0,],
    ]
    floors.append({'height':height, 'data':data})
    # 2nd floor
    height = 3
    data = [
        [-1,-1,-1,-1,],
        [-1,-1,-1, 0,],
        [-1,-1,-1, 0,],
    ]
    floors.append({'height':height, 'data':data})
    result = {
        "floors": floors,
        "position": position,
    }
    return result
