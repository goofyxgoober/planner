import { Handle, Position } from '@xyflow/react';
import ArrowCircleRightIcon from '@mui/icons-material/ArrowCircleRight';

export function CustomNode() {
  return (
    //react-flow__node-default
    <div className="react-flow__node-default">
      <Handle
        position={Position.Left}
        type="source"
        style={{
          background: 'none',
          border: 'none',
          width: '1em',
          height: '1em',
        }}
      >
        <ArrowCircleRightIcon
          style={{
            pointerEvents: 'none',
            fontSize: '1em',
            left: 0,
            position: 'absolute',
          }}
        />
      </Handle>
      Custom Node
    </div>
  );
}

export function GraphNode() {
  
  return (
    //react-flow__node-default
    <div className="react-flow__node-default">
      <Handle
        position={Position.Left}
        type="source"
        style={{
          background: 'none',
          border: 'none',
          width: '1em',
          height: '1em',
        }}
      >
        <ArrowCircleRightIcon
          style={{
            pointerEvents: 'none',
            fontSize: '1em',
            left: 0,
            position: 'absolute',
          }}
        />
      </Handle>
      Custom Node
    </div>
  );
}