using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices.WindowsRuntime;
using Windows.Foundation;
using Windows.Foundation.Collections;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Controls.Primitives;
using Windows.UI.Xaml.Data;
using Windows.UI.Xaml.Input;
using Windows.UI.Xaml.Media;
using Windows.UI.Xaml.Navigation;
using SignIn.Logic.Data;

// The User Control item template is documented at https://go.microsoft.com/fwlink/?LinkId=234236

namespace SignIn.Controls
{
    public sealed partial class AddressControl : UserControl
    {
        public AddressControl()
        {
            InitializeComponent();
        }

        public Address Address
        {
            get { return (Address)GetValue(AddressProperty); }
            set { SetValue(AddressProperty, value); }
        }
        public static readonly DependencyProperty AddressProperty =
            DependencyProperty.Register(nameof(Address), typeof(Address), typeof(AddressControl), new PropertyMetadata(null));
    }
}
