package llr.ui;

import android.os.Bundle;
import android.view.View;
import android.view.accessibility.AccessibilityNodeInfo;
import android.view.accessibility.AccessibilityNodeProvider;

/** Virtual accessibility nodes for a NativeActivity. The native shim fills the tables. */
public final class A11y extends AccessibilityNodeProvider {
    static int[] ids = new int[0];
    static int[] parents = new int[0];
    static int[] roles = new int[0];
    static int[] flags = new int[0];
    static int[] actions = new int[0];
    static float[] rects = new float[0];
    static String[] labels = new String[0];
    static String[] values = new String[0];

    static native void invoke(int id, int action, String text);

    public static void attach(View view) {
        view.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_YES);
        view.setAccessibilityDelegate(new View.AccessibilityDelegate() {
            @Override
            public AccessibilityNodeProvider getAccessibilityNodeProvider(View host) {
                return new A11y();
            }
        });
    }

    public static void setTree(int[] ids, int[] parents, int[] roles, int[] flags, int[] actions,
            float[] rects, String[] labels, String[] values) {
        A11y.ids = ids;
        A11y.parents = parents;
        A11y.roles = roles;
        A11y.flags = flags;
        A11y.actions = actions;
        A11y.rects = rects;
        A11y.labels = labels;
        A11y.values = values;
    }

    private static int indexOf(int id) {
        for (int i = 0; i < ids.length; i++) {
            if (ids[i] == id) {
                return i;
            }
        }
        return -1;
    }

    private static String roleName(int role) {
        switch (role) {
            case 1: return "android.widget.Button";
            case 2: return "android.widget.CheckBox";
            case 3: return "android.widget.Switch";
            case 4: return "android.widget.SeekBar";
            case 5: return "android.widget.EditText";
            case 14: return "android.widget.ImageView";
            default: return "android.view.View";
        }
    }

    @Override
    public AccessibilityNodeInfo createAccessibilityNodeInfo(int virtualId) {
        if (virtualId == View.NO_ID) {
            AccessibilityNodeInfo root = AccessibilityNodeInfo.obtain();
            root.setClassName("android.view.View");
            for (int i = 0; i < ids.length; i++) {
                if (parents[i] == -1) {
                    root.addChild(null, ids[i]);
                }
            }
            return root;
        }
        int at = indexOf(virtualId);
        if (at < 0) {
            return null;
        }
        AccessibilityNodeInfo info = AccessibilityNodeInfo.obtain();
        info.setSource(null, virtualId);
        info.setClassName(roleName(roles[at]));
        info.setContentDescription(labels[at]);
        info.setText(values[at]);
        info.setFocusable((flags[at] & 1) != 0);
        info.setCheckable(roles[at] == 2 || roles[at] == 3);
        info.setChecked((flags[at] & 2) != 0);
        info.setEnabled(true);
        info.setBoundsInParent(new android.graphics.Rect(
                (int) rects[at * 4], (int) rects[at * 4 + 1],
                (int) (rects[at * 4] + rects[at * 4 + 2]),
                (int) (rects[at * 4 + 1] + rects[at * 4 + 3])));
        if ((actions[at] & 1) != 0 || (actions[at] & 8) != 0) {
            info.addAction(AccessibilityNodeInfo.ACTION_CLICK);
        }
        if ((actions[at] & 4) != 0) {
            info.addAction(AccessibilityNodeInfo.ACTION_SET_TEXT);
        }
        info.addAction(AccessibilityNodeInfo.ACTION_ACCESSIBILITY_FOCUS);
        info.setParent(null, parents[at] == -1 ? View.NO_ID : parents[at]);
        for (int i = 0; i < ids.length; i++) {
            if (parents[i] == virtualId) {
                info.addChild(null, ids[i]);
            }
        }
        return info;
    }

    @Override
    public boolean performAction(int virtualId, int action, Bundle args) {
        if (action == AccessibilityNodeInfo.ACTION_CLICK) {
            invoke(virtualId, (actionsOf(virtualId) & 8) != 0 ? 8 : 1, null);
            return true;
        }
        if (action == AccessibilityNodeInfo.ACTION_ACCESSIBILITY_FOCUS) {
            invoke(virtualId, 2, null);
            return true;
        }
        if (action == AccessibilityNodeInfo.ACTION_SET_TEXT) {
            CharSequence text = args == null ? "" : args.getCharSequence(
                    AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE);
            invoke(virtualId, 4, text == null ? "" : text.toString());
            return true;
        }
        return false;
    }

    private static int actionsOf(int id) {
        int at = indexOf(id);
        return at < 0 ? 0 : actions[at];
    }
}
