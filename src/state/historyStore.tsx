import type {Op} from "../types";
import { createStore } from 'zustand'
import type { StoreApi } from 'zustand'

type OpEntry = {op:Op,inverseOp:Op}

type HistoryStoreState = {
    undoStacks: Record<string,Array<OpEntry> | undefined>,
    redoStacks: Record<string,Array<OpEntry> | undefined>,
    pendingOps: Record<string,Array<Op> | undefined>,
}

type HistoryStoreActions = {
    pushToPendingOp: (goalId:string,op:Op) => void,
    popFromPendingOp: (goalId:string)=> Op|undefined,
    pushUndoStack: (goalId:string,op:OpEntry) =>void,
    undo: (goalId:string) => OpEntry | undefined, 
    redo: (goalId:string) => OpEntry | undefined
}

type HistoryStore = HistoryStoreState&HistoryStoreActions;


const createPushToPendingOp = (
    set:StoreApi<HistoryStore>['setState'],
    get:StoreApi<HistoryStore>['getState'],
): HistoryStoreActions['pushToPendingOp'] =>{
    return (goalId,op)=>{
        const currentOps = get().pendingOps[goalId] ?? [];
        set((state)=>({
            pendingOps:{
                ...state.pendingOps,
                [goalId]:{...currentOps,op}
            }
        }));
    }
}

const createPopFromPendingOp = (
    set:StoreApi<HistoryStore>['setState'],
    get:StoreApi<HistoryStore>['getState'],
): HistoryStoreActions['popFromPendingOp'] => {
    return(goalId)=>{
        const currentOps = get().pendingOps[goalId] ?? [];
        if(currentOps.length===0){
            return undefined;
        }
        const poppedOp = currentOps.shift();
        set((state)=>({
            pendingOps:{
                ...state.pendingOps,
                [goalId]:{...currentOps}
            }
        }));
        return poppedOp;
    }
}

const createPushUndoStack = (
    set:StoreApi<HistoryStore>['setState'],
    get:StoreApi<HistoryStore>['getState'],
): HistoryStoreActions['pushUndoStack'] => {
    return (goalId,op)=>{
        const currentOps = get().undoStacks[goalId] ?? [];
        set((state)=>({
            undoStacks:{
                ...state.undoStacks,
                [goalId]:{...currentOps,op}
            }
        }));
    }
}


const createUndo = (
    set:StoreApi<HistoryStore>['setState'],
    get:StoreApi<HistoryStore>['getState'],
): HistoryStoreActions['undo'] => {
    return(goalId)=>{
        const currentOps = get().undoStacks[goalId] ?? [];
        if(currentOps.length===0){
            return undefined;
        }
        const poppedOp = currentOps.pop();
        set((state)=>({
            undoStacks:{
                ...state.undoStacks,
                [goalId]:{...currentOps}
            },
            redoStacks:{
                ...state.redoStacks,
                [goalId]:{...get().redoStacks[goalId]??[],poppedOp}
            }
        }));
        return poppedOp;
    }
}

const createRedo = (
    set:StoreApi<HistoryStore>['setState'],
    get:StoreApi<HistoryStore>['getState'],
): HistoryStoreActions['redo'] => {
    return(goalId)=>{
        const currentOps = get().redoStacks[goalId] ?? [];
        if(currentOps.length===0){
            return undefined;
        }
        const poppedOp = currentOps.pop();
        set((state)=>({
            undoStacks:{
                ...state.undoStacks,
                [goalId]:{...get().undoStacks[goalId]??[],poppedOp}
            },
            redoStacks:{
                ...state.redoStacks,
                [goalId]:{...currentOps}
            }
        }));
        return poppedOp;
    }
}


export const HistoryStore = createStore<HistoryStore>()((set,get)=>({
    undoStacks:{},
    redoStacks:{},
    pendingOps:{},
    pushToPendingOp:createPushToPendingOp(set,get),
    popFromPendingOp:createPopFromPendingOp(set,get),
    pushUndoStack:createPushUndoStack(set,get),
    undo:createUndo(set,get),
    redo:createRedo(set,get)
}))





