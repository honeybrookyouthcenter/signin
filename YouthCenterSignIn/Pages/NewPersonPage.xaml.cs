﻿using System;
using Windows.UI.Popups;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Input;
using Windows.UI.Xaml.Media.Animation;
using YouthCenterSignIn.Logic.Data;

// The Blank Page item template is documented at https://go.microsoft.com/fwlink/?LinkId=234238

namespace YouthCenterSignIn.Pages
{
    /// <summary>
    /// An empty page that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class NewPersonPage : Page
    {
        public NewPersonPage()
        {
            InitializeComponent();

            VisualStateManager.GoToState(this, "PersonInfo", false);
        }

        Person NewPerson { get; } = new Person();

        public string GuardianFullName { get; set; }
        public string GuardianPhoneNumber { get; set; }

        private async void Cancel_Tapped(object sender, TappedRoutedEventArgs e)
        {
            MessageDialog message = new MessageDialog("Are you sure you want to cancel? You'll loose the information you entered.", "Cancel");
            message.Commands.Add(new UICommand("Yes", (_) => ((Frame)Parent).GoBack()));
            message.Commands.Add(new UICommand("No"));

            await message.ShowAsync();
        }

        async void Next_Tapped(object sender, TappedRoutedEventArgs e)
        {
            if (NewPerson.HasValidPersonInfo(out var issues))
                VisualStateManager.GoToState(this, "GuardianInfo", true);
            else
                await new MessageDialog(issues).ShowAsync();
        }

        private void Back_Tapped(object sender, TappedRoutedEventArgs e)
        {
            VisualStateManager.GoToState(this, "PersonInfo", true);
        }

        private void Done_Tapped(object sender, TappedRoutedEventArgs e)
        {
            Logic.Data.DataProvider.Current.AddPerson(NewPerson);
            var parent = (Frame)Parent;
            parent.GoBack(new SlideNavigationTransitionInfo());
            parent.Navigate(typeof(PersonPage), NewPerson);
        }
    }
}
