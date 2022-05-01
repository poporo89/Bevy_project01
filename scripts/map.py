# vim: lsp=4
def test_map():
    result = []
    # 1st floor
    height = 0
    data = [
        [ 0, 1, 2, 3,],
        [ 0,-1,-1, 0,],
        [ 0, 0, 0, 0,],
    ]
    result.append({'height':height, 'data':data})
    # 2nd floor
    height = 3
    data = [
        [-1,-1,-1,-1,],
        [-1,-1,-1, 0,],
        [-1,-1,-1, 0,],
    ]
    result.append({'height':height, 'data':data})
    return result
