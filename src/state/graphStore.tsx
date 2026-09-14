import type { Snapshot, Op, SnapshotVersion } from "../types";
import {invoke} from '@tauri-apps/api/core';
import { createStore } from 'zustand'
import type { StoreApi } from 'zustand'




type GraphStoreState = {
    graphs: Record<string, SnapshotVersion| undefined>,
    errors: Record<string,string | undefined>
}

type GraphStoreActions = {
    loadGraph: (goalId:string)=> Promise<void>
    getGraph: (goalId:string)=> SnapshotVersion | undefined
    setGraph: (goalId:string,spv:SnapshotVersion) => void
    proposeOp: (goalId:string,op:Op) => void
}

type GraphStore = GraphStoreState & GraphStoreActions

const createLoadGraph = (
    set:StoreApi<GraphStore>['setState']
): GraphStoreActions['loadGraph'] =>{
    return async(goalId)=>{
        //const result = await graphSnapshot(goalId);
        set((state)=>({
            errors:{...state.errors,[goalId]:undefined},
        }))
        try{
            const result = await invoke<SnapshotVersion>('get_snapshot', {
                goalId: goalId // Rust receives this as `goal_id`
            });
            set((state)=>({
                graphs: {...state.graphs,[goalId]:result},
            }))
        }
        catch(err){
            set((state)=>({
                errors:{...state.errors,[goalId]:String(err)}
            }))
        }
        
    }
}

const createGetGraph = (
    get:StoreApi<GraphStore>['getState']
): GraphStoreActions['getGraph'] =>{
    return (goalId) => {
        return get().graphs[goalId]
    }
}

const createSetGraph= (
    set:StoreApi<GraphStore>['setState']
): GraphStoreActions['setGraph'] =>{
    return (goalId,spv)=>{
        set((state)=>({
            graphs:{...state.graphs,[goalId]:spv},
        }))
    }
}


const createProposeOp = (
    //set:StoreApi<GraphStore>['setState'],
    get:StoreApi<GraphStore>['getState']
):GraphStoreActions['proposeOp'] =>{
    return async(goalId,op)=>{

        const spv = get().graphs[goalId];
        if(spv==undefined){
            //should be more explicit with what went wrong
            return;
        }
        const result = await invoke<{inverseOp:Array<Op>,version:number,snapshot:Snapshot}>('propose_op', { op, baseVersion:spv['version'] });
        get().setGraph(goalId,{snapshot:result['snapshot'],version:result['version']});
        //need to handle inverse ops
    }
}
export const graphStore = createStore<GraphStore>()((set,get)=>({
    graphs: {},
    errors:{},
    loadGraph:createLoadGraph(set),
    getGraph:createGetGraph(get),
    setGraph:createSetGraph(set),
    proposeOp:createProposeOp(get)

}))

